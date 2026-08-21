use crate::{
    auth::{CurrentUser, issue_token},
    error::ApiError,
    legacy::{HARDWARE_PARTS, PROCESS_ACTIONS, SOFTWARE_TYPES},
    state::AppState,
};
use actix_web::{HttpResponse, web};
use bcrypt::{hash, verify};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow};

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}
#[derive(Deserialize)]
struct Register {
    username: String,
    password: String,
    email: String,
}
#[derive(Deserialize)]
struct ComposeMail {
    to: String,
    subject: String,
    text: String,
}
#[derive(Deserialize)]
struct BankTransfer {
    from_account: i64,
    password: String,
    to_account: i64,
    amount: i64,
}
#[derive(Deserialize)]
struct CreateBankAccount {
    bank_id: i64,
    password: String,
}
#[derive(Deserialize)]
struct MissionCommand {
    action: String,
}
#[derive(Deserialize)]
struct FriendRequest {
    user_id: i64,
}
#[derive(Deserialize)]
struct WikiEdit {
    title: String,
    content: String,
}
#[derive(Deserialize)]
struct ForumTopic {
    title: String,
    content: String,
}
#[derive(Deserialize)]
struct ForumPost {
    content: String,
}
#[derive(Deserialize)]
struct ProfileUpdate {
    biography: String,
    visible: bool,
}
#[derive(Deserialize)]
struct ClanCreate {
    name: String,
    nick: String,
    description: String,
}
#[derive(Deserialize)]
struct ClanJoin {
    message: String,
}
#[derive(Deserialize)]
struct ResearchStart {
    software_id: i64,
}
#[derive(Deserialize)]
struct BitcoinTransfer {
    from: String,
    to: String,
    amount: String,
}
#[derive(Deserialize)]
struct ListEntry {
    ip: i64,
    login: String,
    password: String,
    virus_id: Option<i64>,
}
#[derive(Deserialize)]
struct StartProcess {
    action: u8,
    victim_id: Option<i64>,
    software_id: Option<i64>,
    info: Option<String>,
}
#[derive(Deserialize)]
struct InternetLogin {
    ip: i64,
    user: String,
    password: String,
}
#[derive(Deserialize)]
struct InternetHack {
    ip: i64,
    kind: String,
}
#[derive(Deserialize)]
struct IpBody {
    ip: i64,
}
#[derive(Deserialize)]
struct LogEdit {
    text: String,
}
#[derive(Deserialize)]
struct Transfer {
    software_id: i64,
    direction: String,
}
#[derive(Deserialize)]
struct BuyPart {
    kind: String,
    part: u8,
    account: i64,
}
#[derive(Deserialize)]
struct NewSoftware {
    name: String,
    soft_type: i16,
    size: i32,
    ram: i32,
}
#[derive(Deserialize)]
struct CertificationStart {
    account: Option<i64>,
}
#[derive(Deserialize)]
struct SettingsUpdate {
    email: Option<String>,
    current_password: String,
    new_password: Option<String>,
}
#[derive(Serialize)]
struct Session {
    token: String,
    user: User,
}
#[derive(Serialize)]
struct User {
    id: i64,
    login: String,
    email: String,
    game_ip: i64,
    premium: bool,
}
#[derive(Serialize)]
struct Dashboard {
    user: User,
    money: f64,
    processes: i64,
    unread_mail: i64,
    actions: Vec<CatalogEntry>,
}
#[derive(Serialize)]
struct CatalogEntry {
    name: &'static str,
    id: u8,
}
fn user(row: &PgRow) -> User {
    User {
        id: row.get("id"),
        login: row.get("login"),
        email: row.get("email"),
        game_ip: row.get("game_ip"),
        premium: row.get("premium"),
    }
}
fn actions() -> Vec<CatalogEntry> {
    PROCESS_ACTIONS
        .iter()
        .map(|(name, id)| CatalogEntry { name, id: *id })
        .collect()
}
fn software_types() -> Vec<CatalogEntry> {
    SOFTWARE_TYPES
        .iter()
        .map(|(name, id)| CatalogEntry { name, id: *id })
        .collect()
}

pub fn configure(c: &mut web::ServiceConfig) {
    c.service(
        web::scope("/api")
            //Monitoramiento del backend
            .route("/health", web::get().to(health))

            //Software


















    );
}









async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok"}))
}




