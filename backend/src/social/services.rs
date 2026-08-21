use rand::Rng;
use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        ClanJoinRequest,
        ClanRequestAcceptedResponse,
        ClanRequestResponse,
        ClanResponse,
        CreateClanRequest,
        CreatedResponse,
        FriendAcceptedResponse,
        FriendResponse,
        MyClanResponse,
        RankingResponse,
    },
    repo,
};

//
// ==============================
// RANKING
// ==============================
//

/// El legado mostraba los cien primeros.
const RANKING_SIZE: i64 = 100;

pub async fn ranking(
    pool: &PgPool,
) -> Result<Vec<RankingResponse>, ApiError> {
    let ranking =
        repo::find_ranking(pool, RANKING_SIZE)
            .await?
            .into_iter()
            .map(RankingResponse::from)
            .collect();

    Ok(ranking)
}

//
// ==============================
// FRIENDS
// ==============================
//

pub async fn friends(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<FriendResponse>, ApiError> {
    let friends =
        repo::find_friends(pool, user_id)
            .await?
            .into_iter()
            .map(FriendResponse::from)
            .collect();

    Ok(friends)
}

pub async fn request_friend(
    pool: &PgPool,
    user_id: i64,
    target_id: i64,
) -> Result<CreatedResponse, ApiError> {
    if target_id == user_id {
        return Err(ApiError::BadRequest(
            "you can not add yourself".into(),
        ));
    }

    let id = repo::insert_friend_request(
        pool, target_id, user_id,
    )
    .await?
    .ok_or_else(|| {
        ApiError::BadRequest(
            "request already exists or user missing"
                .into(),
        )
    })?;

    Ok(CreatedResponse { id })
}

/// Aceptar consume la solicitud y crea la amistad en las dos direcciones, todo junto.
pub async fn accept_friend(
    pool: &PgPool,
    user_id: i64,
    request_id: i64,
) -> Result<FriendAcceptedResponse, ApiError> {
    let mut tx = pool.begin().await?;

    let friend_id = repo::take_friend_request(
        &mut tx,
        request_id,
        user_id,
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    repo::insert_friendship(
        &mut tx,
        user_id,
        friend_id,
    )
    .await?;

    tx.commit().await?;

    Ok(FriendAcceptedResponse { friend_id })
}

//
// ==============================
// CLAN LIST
// ==============================
//

pub async fn clans(
    pool: &PgPool,
) -> Result<Vec<ClanResponse>, ApiError> {
    let clans = repo::find_clans(pool)
        .await?
        .into_iter()
        .map(ClanResponse::from)
        .collect();

    Ok(clans)
}

pub async fn my_clan(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<MyClanResponse>, ApiError> {
    let Some(clan) =
        repo::find_my_clan(pool, user_id).await?
    else {
        return Ok(None);
    };

    let members =
        repo::find_members(pool, clan.id).await?;

    Ok(Some(MyClanResponse::new(clan, members)))
}

//
// ==============================
// CLAN CREATION
// ==============================
//

const NAME_MAX: usize = 25;
const NICK_LEN: usize = 3;
const DESCRIPTION_MAX: usize = 3000;

pub async fn create_clan(
    pool: &PgPool,
    user_id: i64,
    input: CreateClanRequest,
) -> Result<CreatedResponse, ApiError> {
    let name = input.name.trim();
    let nick = input.nick.trim().to_uppercase();

    let nick_ok = nick.len() == NICK_LEN
        && nick.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
        });

    if name.is_empty()
        || name.len() > NAME_MAX
        || !nick_ok
        || input.description.len()
            > DESCRIPTION_MAX
    {
        return Err(ApiError::BadRequest(
            "invalid clan data".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    if repo::in_any_clan_tx(&mut tx, user_id)
        .await?
    {
        return Err(ApiError::BadRequest(
            "already in a clan".into(),
        ));
    }

    let clan_ip = rand::rng()
        .random_range(1_i64..=u32::MAX as i64);

    let id = repo::insert_clan(
        &mut tx,
        clan_ip,
        name,
        &nick,
        input.description.trim(),
        user_id,
    )
    .await?;

    repo::insert_founder(&mut tx, id, user_id)
        .await?;

    tx.commit().await?;

    Ok(CreatedResponse { id })
}

//
// ==============================
// JOIN REQUESTS
// ==============================
//

pub async fn request_join(
    pool: &PgPool,
    user_id: i64,
    clan_id: i64,
    input: ClanJoinRequest,
) -> Result<CreatedResponse, ApiError> {
    if input.message.len() > DESCRIPTION_MAX {
        return Err(ApiError::BadRequest(
            "request message is too long".into(),
        ));
    }

    if repo::in_any_clan(pool, user_id).await? {
        return Err(ApiError::BadRequest(
            "already in a clan".into(),
        ));
    }

    let id = repo::insert_join_request(
        pool,
        clan_id,
        user_id,
        input.message.trim(),
    )
    .await?
    .ok_or_else(|| {
        ApiError::BadRequest(
            "request already exists or clan missing"
                .into(),
        )
    })?;

    Ok(CreatedResponse { id })
}

pub async fn requests(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<ClanRequestResponse>, ApiError> {
    let requests =
        repo::find_requests_for_manager(
            pool, user_id,
        )
        .await?
        .into_iter()
        .map(ClanRequestResponse::from)
        .collect();

    Ok(requests)
}

pub async fn accept_request(
    pool: &PgPool,
    user_id: i64,
    request_id: i64,
) -> Result<ClanRequestAcceptedResponse, ApiError>
{
    let mut tx = pool.begin().await?;

    let request =
        repo::lock_request_for_manager(
            &mut tx, request_id, user_id,
        )
        .await?
        .ok_or(ApiError::NotFound)?;

    let members = repo::count_members(
        &mut tx,
        request.clan_id,
    )
    .await?;

    if members >= request.slots_total as i64 {
        return Err(ApiError::BadRequest(
            "clan is full".into(),
        ));
    }

    repo::insert_member(
        &mut tx,
        request.clan_id,
        request.user_id,
    )
    .await?;

    repo::delete_request(&mut tx, request_id)
        .await?;

    tx.commit().await?;

    Ok(ClanRequestAcceptedResponse {
        clan_id: request.clan_id,
        user_id: request.user_id,
    })
}
