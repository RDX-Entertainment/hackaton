use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::models::{
    ClanMemberRow,
    ClanRequestRow,
    ClanRow,
    FriendRow,
    MyClanRow,
    RankingRow,
};

//
// ==============================
// RANKING
// ==============================
//

#[derive(Debug, Serialize)]
pub struct RankingResponse {
    pub rank: i32,
    pub user_id: i64,
    pub login: String,
}

impl From<RankingRow> for RankingResponse {
    fn from(value: RankingRow) -> Self {
        Self {
            rank: value.rank,
            user_id: value.id,
            login: value.login,
        }
    }
}

//
// ==============================
// FRIENDS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct FriendResponse {
    pub id: i64,
    pub login: String,
}

impl From<FriendRow> for FriendResponse {
    fn from(value: FriendRow) -> Self {
        Self {
            id: value.id,
            login: value.login,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FriendRequestRequest {
    pub user_id: i64,
}

#[derive(Debug, Serialize)]
pub struct CreatedResponse {
    pub id: i64,
}

#[derive(Debug, Serialize)]
pub struct FriendAcceptedResponse {
    pub friend_id: i64,
}

//
// ==============================
// CLAN LIST
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ClanResponse {
    pub id: i64,
    pub ip: i64,
    pub name: String,
    pub nick: String,
    pub description: String,
    pub slots: i16,
    pub members: i64,
    pub money: i64,
    pub power: i64,
}

impl From<ClanRow> for ClanResponse {
    fn from(value: ClanRow) -> Self {
        Self {
            id: value.id,
            ip: value.clan_ip,
            name: value.name,
            nick: value.nick,
            description: value.description,
            slots: value.slots_total,
            members: value.members,
            money: value.money,
            power: value.power,
        }
    }
}

//
// ==============================
// OWN CLAN
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ClanMemberResponse {
    pub id: i64,
    pub login: String,
    pub auth_level: i16,
}

impl From<ClanMemberRow> for ClanMemberResponse {
    fn from(value: ClanMemberRow) -> Self {
        Self {
            id: value.id,
            login: value.login,
            auth_level: value.auth_level,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MyClanResponse {
    pub id: i64,
    pub name: String,
    pub nick: String,
    pub description: String,
    pub slots: i16,
    pub money: i64,
    pub power: i64,
    pub auth_level: i16,
    pub members: Vec<ClanMemberResponse>,
}

impl MyClanResponse {
    pub fn new(
        clan: MyClanRow,
        members: Vec<ClanMemberRow>,
    ) -> Self {
        Self {
            id: clan.id,
            name: clan.name,
            nick: clan.nick,
            description: clan.description,
            slots: clan.slots_total,
            money: clan.money,
            power: clan.power,
            auth_level: clan.auth_level,
            members: members
                .into_iter()
                .map(ClanMemberResponse::from)
                .collect(),
        }
    }
}

//
// ==============================
// CLAN WRITING
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct CreateClanRequest {
    pub name: String,
    pub nick: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ClanJoinRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ClanRequestResponse {
    pub id: i64,
    pub login: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

impl From<ClanRequestRow> for ClanRequestResponse {
    fn from(value: ClanRequestRow) -> Self {
        Self {
            id: value.id,
            login: value.login,
            message: value.message,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ClanRequestAcceptedResponse {
    pub clan_id: i64,
    pub user_id: i64,
}
