use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct DashboardUserRow {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub game_ip: i64,
    pub premium: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct SettingsRow {
    pub login: String,
    pub email: String,
    pub premium: bool,
    pub last_login: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ProfileRow {
    pub id: i64,
    pub login: String,
    pub game_ip: i64,
    pub biography: String,
    pub visible: bool,
    pub reputation: i64,
    pub rank: i32,
    pub mission_count: i32,
    pub hack_count: i32,
    pub money_earned: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PlayerLogEntry {
    pub id: i64,
    pub text: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CompromisedListEntry {
    pub id: i64,
    pub ip: i64,
    pub login: String,
    pub hacked_at: DateTime<Utc>,
    pub virus_id: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct CertificationDefinition {
    pub level: i16,
    pub name: &'static str,
    pub description: &'static str,
    pub price: i64,
}

impl CertificationDefinition {
    pub fn by_level(level: i16) -> Option<Self> {
        match level {
            1 => Some(Self {
                level,
                name: "Basic tutorial",
                description: "Follow this tutorial to learn how to play",
                price: 0,
            }),

            2 => Some(Self {
                level,
                name: "Hacking 101",
                description: "Learn the basics of hacking",
                price: 0,
            }),

            3 => Some(Self {
                level,
                name: "Intermediate hacking",
                description: "Improve your hacking techniques",
                price: 50,
            }),

            4 => Some(Self {
                level,
                name: "Advanced hacking",
                description: "Meet the wonders of DDoS world",
                price: 200,
            }),

            5 => Some(Self {
                level,
                name: "DDoS security",
                description: "Learn to protect yourself from DDoS",
                price: 500,
            }),

            _ => None,
        }
    }

    pub fn all() -> impl Iterator<Item = Self> {
        (1..=5).filter_map(Self::by_level)
    }
}