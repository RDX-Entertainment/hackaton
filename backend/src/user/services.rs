use bcrypt::{
    hash,
    verify,
};

use serde_json::Value;
use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        CertificationCompletedResponse,
        CertificationItemResponse,
        CertificationStartRequest,
        CertificationStartedResponse,
        CertificationsResponse,
        CreateCompromisedListEntryRequest,
        CreatedIdResponse,
        DashboardResponse,
        DashboardUserResponse,
        ProfileResponse,
        ProfileUpdate,
        SavedResponse,
        SettingsResponse,
        SettingsUpdate,
    },
    models::{
        CertificationDefinition,
        CompromisedListEntry,
        PlayerLogEntry,
    },
    repo,
};

//
// ==============================
// DASHBOARD
// ==============================
//

pub async fn dashboard(
    pool: &PgPool,
    user_id: i64,
    actions: Value,
) -> Result<DashboardResponse, ApiError> {
    let user = repo::find_dashboard_user(pool, user_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let money = repo::total_money(pool, user_id)
        .await?
        .unwrap_or(0);

    let processes = repo::process_count(
        pool,
        user_id,
    )
    .await?;

    let unread_mail = repo::unread_mail_count(
        pool,
        user_id,
    )
    .await?;

    Ok(DashboardResponse {
        user: DashboardUserResponse::from(user),
        money: money as f64,
        processes,
        unread_mail,
        actions,
    })
}

//
// ==============================
// LOGS
// ==============================
//

pub async fn player_log(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<PlayerLogEntry>, ApiError> {
    let entries = repo::player_log(
        pool,
        user_id,
    )
    .await?;

    Ok(entries)
}

//
// ==============================
// SETTINGS
// ==============================
//

pub async fn settings(
    pool: &PgPool,
    user_id: i64,
) -> Result<SettingsResponse, ApiError> {
    let settings = repo::find_settings(
        pool,
        user_id,
    )
    .await?;

    Ok(settings.into())
}

pub async fn save_settings(
    pool: &PgPool,
    user_id: i64,
    input: SettingsUpdate,
) -> Result<SavedResponse, ApiError> {
    if input.current_password.is_empty() {
        return Err(ApiError::BadRequest(
            "current password is required".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    let current_password_hash = repo::lock_password(
        &mut tx,
        user_id,
    )
    .await?;

    let valid_password = verify(
        &input.current_password,
        &current_password_hash,
    )
    .map_err(|_| ApiError::Unauthorized)?;

    if !valid_password {
        return Err(ApiError::Unauthorized);
    }

    //
    // Email
    //

    if let Some(email) = input.email.as_deref() {
        let email = email.trim();

        if !email.contains('@') || email.len() > 50 {
            return Err(ApiError::BadRequest(
                "invalid email".into(),
            ));
        }

        let exists = repo::email_exists_for_other_user(
            &mut tx,
            email,
            user_id,
        )
        .await?;

        if exists {
            return Err(ApiError::BadRequest(
                "email already exists".into(),
            ));
        }

        repo::update_email(
            &mut tx,
            user_id,
            email,
        )
        .await?;
    }

    //
    // Password
    //

    if let Some(new_password) = input.new_password.as_deref() {
        if new_password.len() < 8 {
            return Err(ApiError::BadRequest(
                "new password must have 8 characters".into(),
            ));
        }

        let password_hash = hash(
            new_password,
            13,
        )
        .map_err(|_| {
            ApiError::BadRequest(
                "cannot hash password".into(),
            )
        })?;

        repo::update_password(
            &mut tx,
            user_id,
            &password_hash,
        )
        .await?;
    }

    repo::insert_account_settings_log(
        &mut tx,
        user_id,
    )
    .await?;

    tx.commit().await?;

    Ok(SavedResponse {
        saved: true,
    })
}

//
// ==============================
// CERTIFICATIONS
// ==============================
//

pub async fn certifications(
    pool: &PgPool,
    user_id: i64,
) -> Result<CertificationsResponse, ApiError> {
    let level = repo::certification_level(
        pool,
        user_id,
    )
    .await?;

    let learning = repo::user_learning(
        pool,
        user_id,
    )
    .await?;

    let current = repo::active_puzzle(
        pool,
        user_id,
    )
    .await?;

    let items = CertificationDefinition::all()
        .map(|certification| {
            CertificationItemResponse {
                level: certification.level,
                name: certification.name,
                description: certification.description,
                price: certification.price,

                completed:
                    level >= certification.level,

                available:
                    learning
                        && level
                            == certification.level - 1,

                learning:
                    !learning
                        && current
                            == Some(
                                certification.level as i64,
                            ),
            }
        })
        .collect();

    Ok(CertificationsResponse {
        level,
        items,
    })
}

pub async fn start_certification(
    pool: &PgPool,
    user_id: i64,
    level: i16,
    input: CertificationStartRequest,
) -> Result<CertificationStartedResponse, ApiError> {
    let certification =
        CertificationDefinition::by_level(level)
            .ok_or_else(|| {
                ApiError::BadRequest(
                    "invalid certification".into(),
                )
            })?;

    let mut tx = pool.begin().await?;

    let current =
        repo::lock_certification_level(
            &mut tx,
            user_id,
        )
        .await?;

    let learning =
        repo::lock_learning(
            &mut tx,
            user_id,
        )
        .await?;

    //
    // Certification progression
    //

    if current != level - 1 || !learning {
        return Err(ApiError::BadRequest(
            "certification is not available".into(),
        ));
    }

    //
    // Payment
    //

    if certification.price > 0 {
        let account = input.account.ok_or_else(|| {
            ApiError::BadRequest(
                "bank account required".into(),
            )
        })?;

        let cash = repo::lock_bank_cash(
            &mut tx,
            user_id,
            account,
        )
        .await?
        .unwrap_or(0);

        if cash < certification.price {
            return Err(ApiError::BadRequest(
                "insufficient funds".into(),
            ));
        }

        repo::debit_bank_account(
            &mut tx,
            user_id,
            account,
            certification.price,
        )
        .await?;
    }

    //
    // Start learning
    //

    repo::set_learning(
        &mut tx,
        user_id,
        false,
    )
    .await?;

    repo::set_puzzle(
        &mut tx,
        user_id,
        level as i64,
    )
    .await?;

    tx.commit().await?;

    Ok(CertificationStartedResponse {
        level,
        started: true,
    })
}

pub async fn complete_certification(
    pool: &PgPool,
    user_id: i64,
    level: i16,
) -> Result<CertificationCompletedResponse, ApiError> {
    CertificationDefinition::by_level(level)
        .ok_or_else(|| {
            ApiError::BadRequest(
                "invalid certification".into(),
            )
        })?;

    let mut tx = pool.begin().await?;

    let active =
        repo::lock_active_puzzle(
            &mut tx,
            user_id,
        )
        .await?;

    if active != Some(level as i64) {
        return Err(ApiError::BadRequest(
            "certification is not being learned".into(),
        ));
    }

    repo::set_certification_level(
        &mut tx,
        user_id,
        level,
    )
    .await?;

    repo::set_learning(
        &mut tx,
        user_id,
        true,
    )
    .await?;

    repo::set_puzzle(
        &mut tx,
        user_id,
        0,
    )
    .await?;

    tx.commit().await?;

    Ok(CertificationCompletedResponse {
        level,
        completed: true,
    })
}

//
// ==============================
// PROFILE
// ==============================
//

pub async fn profile(
    pool: &PgPool,
    user_id: i64,
) -> Result<ProfileResponse, ApiError> {
    let profile = repo::find_profile(
        pool,
        user_id,
    )
    .await?;

    Ok(profile.into())
}

pub async fn save_profile(
    pool: &PgPool,
    user_id: i64,
    input: ProfileUpdate,
) -> Result<SavedResponse, ApiError> {
    let biography = input.biography.trim();

    if biography.len() > 3000 {
        return Err(ApiError::BadRequest(
            "biography is too long".into(),
        ));
    }

    repo::upsert_profile(
        pool,
        user_id,
        biography,
        input.visible,
    )
    .await?;

    Ok(SavedResponse {
        saved: true,
    })
}

//
// ==============================
// COMPROMISED LISTS
// ==============================
//

pub async fn compromised_lists(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<CompromisedListEntry>, ApiError> {
    let entries = repo::compromised_lists(
        pool,
        user_id,
    )
    .await?;

    Ok(entries)
}

pub async fn add_compromised_list(
    pool: &PgPool,
    user_id: i64,
    input: CreateCompromisedListEntryRequest,
) -> Result<CreatedIdResponse, ApiError> {
    let login = input.login.trim();

    if login.is_empty()
        || login.len() > 15
        || input.password.is_empty()
        || input.password.len() > 60
        || input.ip <= 0
    {
        return Err(ApiError::BadRequest(
            "invalid compromised account".into(),
        ));
    }

    let id = repo::insert_compromised_list(
        pool,
        user_id,
        input.ip,
        login,
        &input.password,
        input.virus_id.unwrap_or(0),
    )
    .await?;

    Ok(CreatedIdResponse {
        id,
    })
}

pub async fn delete_compromised_list(
    pool: &PgPool,
    user_id: i64,
    id: i64,
) -> Result<(), ApiError> {
    let deleted =
        repo::delete_compromised_list(
            pool,
            user_id,
            id,
        )
        .await?;

    if !deleted {
        return Err(ApiError::NotFound);
    }

    Ok(())
}