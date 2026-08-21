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
            .route("/health", web::get().to(health))
            .route("/auth/login", web::post().to(login))
            .route("/auth/register", web::post().to(register))
            .route("/me", web::get().to(me))
            .route("/dashboard", web::get().to(dashboard))
            .route("/game/process-actions", web::get().to(process_actions))
            .route("/game/software-types", web::get().to(software_type_catalog))
            .route("/software", web::get().to(software))
            .route("/hardware", web::get().to(hardware))
            .route("/hardware/shop", web::get().to(hardware_shop))
            .route("/hardware/shop", web::post().to(buy_hardware))
            .route("/software", web::post().to(create_software))
            .route("/processes", web::get().to(processes))
            .route("/processes", web::post().to(start_process))
            .route("/processes/{id}", web::delete().to(delete_process))
            .route("/processes/{id}/pause", web::post().to(pause_process))
            .route("/processes/{id}/resume", web::post().to(resume_process))
            .route("/mails", web::get().to(mails))
            .route("/mails", web::post().to(compose_mail))
            .route("/mails/{id}", web::get().to(mail))
            .route("/mails/{id}", web::delete().to(delete_mail))
            .route("/missions", web::get().to(missions))
            .route("/missions/{id}", web::post().to(mission_command))
            .route("/finances/accounts", web::get().to(bank_accounts))
            .route("/finances/accounts", web::post().to(create_bank_account))
            .route("/finances/transfers", web::post().to(bank_transfer))
            .route("/ranking/users", web::get().to(ranking))
            .route("/friends", web::get().to(friends))
            .route("/friends/requests", web::post().to(request_friend))
            .route(
                "/friends/requests/{id}/accept",
                web::post().to(accept_friend),
            )
            .route("/wiki", web::get().to(wiki_index))
            .route("/wiki/{slug}", web::get().to(wiki_page))
            .route("/wiki/{slug}", web::put().to(save_wiki_page))
            .route("/wiki/{slug}/revisions", web::get().to(wiki_revisions))
            .route("/forum/topics", web::get().to(forum_topics))
            .route("/forum/topics", web::post().to(create_forum_topic))
            .route("/forum/topics/{id}", web::get().to(forum_topic))
            .route(
                "/forum/topics/{id}/posts",
                web::post().to(create_forum_post),
            )
            .route("/profile", web::get().to(profile))
            .route("/profile", web::put().to(save_profile))
            .route("/clans", web::get().to(clans))
            .route("/clans", web::post().to(create_clan))
            .route("/clans/mine", web::get().to(my_clan))
            .route("/clans/requests", web::get().to(clan_requests))
            .route("/clans/{id}/join", web::post().to(request_clan_join))
            .route(
                "/clans/requests/{id}/accept",
                web::post().to(accept_clan_request),
            )
            .route("/research", web::get().to(research))
            .route("/research", web::post().to(start_research))
            .route("/bitcoin/wallets", web::get().to(bitcoin_wallets))
            .route("/bitcoin/wallets", web::post().to(create_bitcoin_wallet))
            .route("/bitcoin/transfers", web::post().to(bitcoin_transfer))
            .route("/lists", web::get().to(compromised_lists))
            .route("/lists", web::post().to(add_compromised_list))
            .route("/lists/{id}", web::delete().to(delete_compromised_list))
            .route("/certifications", web::get().to(certifications))
            .route(
                "/certifications/{level}/start",
                web::post().to(start_certification),
            )
            .route(
                "/certifications/{level}/complete",
                web::post().to(complete_certification),
            )
            .route("/logs", web::get().to(player_log))
            .route("/stats", web::get().to(server_stats))
            .route("/settings", web::get().to(settings))
            .route("/settings", web::put().to(save_settings))
            .route("/news", web::get().to(news))
            .route("/internet/targets", web::get().to(internet_targets))
            .route("/internet/history", web::get().to(internet_history))
            .route("/internet/bookmarks", web::get().to(internet_bookmarks))
            .route("/internet/bookmarks", web::post().to(add_bookmark))
            .route("/internet/bookmarks/{ip}", web::delete().to(remove_bookmark))
            .route("/internet/login", web::post().to(internet_login))
            .route("/internet/hack", web::post().to(internet_hack))
            .route("/internet/{ip}/software", web::get().to(remote_software))
            .route("/internet/{ip}/software", web::post().to(transfer_software))
            .route("/internet/{ip}/software/{soft}", web::delete().to(delete_remote_software))
            .route("/internet/{ip}/log", web::get().to(remote_log))
            .route("/internet/{ip}/log", web::put().to(edit_remote_log))
            .route("/internet/{ip}", web::get().to(internet_navigate))
            .route("/pages/{page}", web::get().to(page_data)),
    );
}
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok"}))
}
async fn login(s: web::Data<AppState>, i: web::Json<Login>) -> Result<HttpResponse, ApiError> {
    let r = sqlx::query("SELECT id,login,email,game_ip,premium,password FROM users WHERE login=$1")
        .bind(i.username.trim())
        .fetch_optional(&s.pool)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if !verify(&i.password, &r.get::<String, _>("password")).map_err(|_| ApiError::Unauthorized)? {
        return Err(ApiError::Unauthorized);
    }
    let id = r.get("id");
    sqlx::query("UPDATE users SET last_login=now() WHERE id=$1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    Ok(HttpResponse::Ok().json(Session {
        token: issue_token(id, &s.jwt_secret)?,
        user: user(&r),
    }))
}
async fn register(
    s: web::Data<AppState>,
    i: web::Json<Register>,
) -> Result<HttpResponse, ApiError> {
    let name = i.username.trim();
    if !(1..=15).contains(&name.len())
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        || i.password.is_empty()
        || !i.email.contains('@')
    {
        return Err(ApiError::BadRequest("invalid registration data".into()));
    }
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE login=$1 OR email=$2)")
            .bind(name)
            .bind(i.email.trim())
            .fetch_one(&s.pool)
            .await?;
    if exists {
        return Err(ApiError::BadRequest(
            "username or email already exists".into(),
        ));
    }
    let ip = rand::rng().random_range(1_i64..=u32::MAX as i64);
    let pass: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let pwd =
        hash(&i.password, 13).map_err(|_| ApiError::BadRequest("cannot hash password".into()))?;
    let mut tx = s.pool.begin().await?;
    let id:i64=sqlx::query_scalar("INSERT INTO users(login,password,email,game_pass,game_ip,real_ip,home_ip) VALUES($1,$2,$3,$4,$5,$5,$5) RETURNING id").bind(name).bind(pwd).bind(i.email.trim()).bind(pass).bind(ip).fetch_one(&mut *tx).await?;
    for q in [
        "INSERT INTO users_stats(uid) VALUES($1)",
        "INSERT INTO hardware(user_id,name) VALUES($1,'Server #1')",
        "INSERT INTO log(user_id,text) VALUES($1,'localhost installed current operating system')",
        "INSERT INTO cache(user_id) VALUES($1)",
        "INSERT INTO cache_profile(user_id) VALUES($1)",
        "INSERT INTO ranking_user(user_id,rank) VALUES($1,-1)",
        "INSERT INTO certifications(user_id) VALUES($1)",
        "INSERT INTO users_puzzle(user_id) VALUES($1)",
        "INSERT INTO users_learning(user_id) VALUES($1)",
        "INSERT INTO users_language(user_id) VALUES($1)",
        "INSERT INTO player_profiles(user_id) VALUES($1)",
    ] {
        sqlx::query(q).bind(id).execute(&mut *tx).await?;
    }
    let r = sqlx::query("SELECT id,login,email,game_ip,premium FROM users WHERE id=$1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(Session {
        token: issue_token(id, &s.jwt_secret)?,
        user: user(&r),
    }))
}
async fn me(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let r = sqlx::query("SELECT id,login,email,game_ip,premium FROM users WHERE id=$1")
        .bind(u.0)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(HttpResponse::Ok().json(user(&r)))
}
async fn dashboard(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let r = sqlx::query("SELECT id,login,email,game_ip,premium FROM users WHERE id=$1")
        .bind(u.0)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(ApiError::NotFound)?;
    let money: Option<i64> =
        sqlx::query_scalar("SELECT SUM(cash) FROM bank_accounts WHERE bank_user=$1")
            .bind(u.0)
            .fetch_one(&s.pool)
            .await?;
    let processes = sqlx::query_scalar("SELECT COUNT(*) FROM processes WHERE creator_id=$1")
        .bind(u.0)
        .fetch_one(&s.pool)
        .await?;
    let unread_mail = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mails WHERE recipient_id=$1 AND NOT is_read AND NOT is_deleted",
    )
    .bind(u.0)
    .fetch_one(&s.pool)
    .await?;
    Ok(HttpResponse::Ok().json(Dashboard {
        user: user(&r),
        money: money.unwrap_or(0) as f64,
        processes,
        unread_mail,
        actions: actions(),
    }))
}
async fn process_actions(_: CurrentUser) -> HttpResponse {
    HttpResponse::Ok().json(actions())
}
async fn software_type_catalog(_: CurrentUser) -> HttpResponse {
    HttpResponse::Ok().json(software_types())
}
async fn rows(s: &AppState, u: i64, sql: &str) -> Result<Vec<PgRow>, ApiError> {
    Ok(sqlx::query(sql).bind(u).fetch_all(&s.pool).await?)
}
async fn software(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let v=rows(&s,u.0,"SELECT s.id,s.soft_name,s.soft_version,s.soft_size,s.soft_ram,s.soft_type,s.soft_hidden,s.soft_hidden_with,s.is_folder,(r.soft_id IS NOT NULL) AS running FROM software s LEFT JOIN software_running r ON r.soft_id=s.id WHERE s.user_id=$1 AND NOT s.is_npc ORDER BY s.soft_name").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"name":r.get::<String,_>("soft_name"),"version":r.get::<i32,_>("soft_version"),"size":r.get::<i32,_>("soft_size"),"ram":r.get::<i32,_>("soft_ram"),"type":r.get::<i16,_>("soft_type"),"hidden":r.get::<bool,_>("soft_hidden"),"hidden_with":r.get::<i32,_>("soft_hidden_with"),"running":r.get::<bool,_>("running"),"folder":r.get::<bool,_>("is_folder")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn hardware(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let v=rows(&s,u.0,"SELECT server_id,name,cpu,hdd,ram,net FROM hardware WHERE user_id=$1 AND NOT is_npc ORDER BY server_id").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("server_id"),"name":r.get::<String,_>("name"),"cpu":r.get::<f32,_>("cpu"),"hdd":r.get::<f32,_>("hdd"),"ram":r.get::<f32,_>("ram"),"net":r.get::<f32,_>("net")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn processes(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let v=rows(&s,u.0,"SELECT pid,victim_id,action,info,info_str,time_ideal,time_worked,is_paused,completed_at FROM processes WHERE creator_id=$1 ORDER BY time_start DESC").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("pid"),"victim_id":r.get::<i64,_>("victim_id"),"action":r.get::<i16,_>("action"),"info":r.get::<String,_>("info"),"details":r.get::<String,_>("info_str"),"ideal_seconds":r.get::<i32,_>("time_ideal"),"worked_seconds":r.get::<i32,_>("time_worked"),"paused":r.get::<bool,_>("is_paused"),"completed":r.get::<Option<chrono::DateTime<chrono::Utc>>,_>("completed_at").is_some()})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn start_process(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<StartProcess>,
) -> Result<HttpResponse, ApiError> {
    if !PROCESS_ACTIONS.iter().any(|(_, id)| *id == i.action) {
        return Err(ApiError::BadRequest("unsupported process action".into()));
    }
    let info = i.info.as_deref().unwrap_or("").trim();
    if info.len() > 30 {
        return Err(ApiError::BadRequest("process info is too long".into()));
    }
    let soft = i.software_id.unwrap_or(0);
    if soft != 0 {
        let owned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc)",
        )
        .bind(soft)
        .bind(u.0)
        .fetch_one(&s.pool)
        .await?;
        if !owned {
            return Err(ApiError::NotFound);
        }
    }
    let seconds: i32 = match i.action {
        1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 13 | 14 => 30,
        10 | 11 | 12 | 15 | 16 | 17 | 22 | 23 => 60,
        18 | 19 | 20 | 24 | 25 | 26 | 27 | 28 => 120,
        _ => 60,
    };
    let duplicate:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM processes WHERE creator_id=$1 AND action=$2 AND victim_id=$3 AND soft_id=$4 AND info=$5 AND completed_at IS NULL)").bind(u.0).bind(i.action as i16).bind(i.victim_id.unwrap_or(0)).bind(soft).bind(info).fetch_one(&s.pool).await?;
    if duplicate {
        return Err(ApiError::BadRequest(
            "identical process already active".into(),
        ));
    }
    let id:i64=sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal) VALUES($1,$2,$3,$4,$5,$5,now()+($6::integer*interval '1 second'),$6) RETURNING pid").bind(u.0).bind(i.victim_id.unwrap_or(0)).bind(i.action as i16).bind(soft).bind(info).bind(seconds).fetch_one(&s.pool).await?;
    Ok(HttpResponse::Created()
        .json(serde_json::json!({"id":id,"action":i.action,"seconds":seconds})))
}
async fn delete_process(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    if sqlx::query("DELETE FROM processes WHERE pid=$1 AND creator_id=$2")
        .bind(*id)
        .bind(u.0)
        .execute(&s.pool)
        .await?
        .rows_affected()
        == 0
    {
        return Err(ApiError::NotFound);
    }
    Ok(HttpResponse::NoContent().finish())
}
async fn pause_process(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let mut tx = s.pool.begin().await?;
    let end:Option<chrono::DateTime<chrono::Utc>>=sqlx::query_scalar("SELECT time_end FROM processes WHERE pid=$1 AND creator_id=$2 AND NOT is_paused FOR UPDATE").bind(*id).bind(u.0).fetch_optional(&mut *tx).await?;
    let end = end.ok_or(ApiError::NotFound)?;
    let seconds: i32 = sqlx::query_scalar(
        "SELECT GREATEST(0,EXTRACT(EPOCH FROM ($1::timestamptz-now()))::integer)",
    )
    .bind(end)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("UPDATE processes SET is_paused=true WHERE pid=$1; INSERT INTO processes_paused(pid,time_left,user_id) VALUES($1,$2,$3) ON CONFLICT(pid) DO UPDATE SET time_left=EXCLUDED.time_left,user_id=EXCLUDED.user_id,time_paused=now()").bind(*id).bind(seconds).bind(u.0).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"id":*id,"paused":true,"seconds_left":seconds})))
}
async fn resume_process(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let mut tx = s.pool.begin().await?;
    let seconds:Option<i32>=sqlx::query_scalar("SELECT pp.time_left FROM processes_paused pp JOIN processes p ON p.pid=pp.pid WHERE pp.pid=$1 AND pp.user_id=$2 AND p.creator_id=$2 AND p.is_paused FOR UPDATE").bind(*id).bind(u.0).fetch_optional(&mut *tx).await?;
    let seconds = seconds.ok_or(ApiError::NotFound)?;
    sqlx::query("UPDATE processes SET is_paused=false,time_start=now(),time_end=now()+($1::integer * interval '1 second') WHERE pid=$2; DELETE FROM processes_paused WHERE pid=$2").bind(seconds).bind(*id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(
        HttpResponse::Ok()
            .json(serde_json::json!({"id":*id,"paused":false,"seconds_left":seconds})),
    )
}
async fn mails(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let v=rows(&s,u.0,"SELECT id,sender_id,recipient_id,type,subject,text,is_read FROM mails WHERE (sender_id=$1 OR recipient_id=$1) AND NOT is_deleted ORDER BY date_sent DESC").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"from":r.get::<i64,_>("sender_id"),"to":r.get::<i64,_>("recipient_id"),"type":r.get::<i16,_>("type"),"subject":r.get::<String,_>("subject"),"text":r.get::<String,_>("text"),"read":r.get::<bool,_>("is_read")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn mail(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let r=sqlx::query("SELECT id,sender_id,recipient_id,type,subject,text,is_read,is_deleted FROM mails WHERE id=$1 AND (sender_id=$2 OR recipient_id=$2)").bind(*id).bind(u.0).fetch_optional(&s.pool).await?.ok_or(ApiError::NotFound)?;
    if r.get::<bool, _>("is_deleted") && r.get::<i64, _>("sender_id") != u.0 {
        return Err(ApiError::NotFound);
    }
    if r.get::<i64, _>("recipient_id") == u.0 {
        sqlx::query("UPDATE mails SET is_read=true WHERE id=$1")
            .bind(*id)
            .execute(&s.pool)
            .await?;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"id":r.get::<i64,_>("id"),"from":r.get::<i64,_>("sender_id"),"to":r.get::<i64,_>("recipient_id"),"type":r.get::<i16,_>("type"),"subject":r.get::<String,_>("subject"),"text":r.get::<String,_>("text"),"read":true})))
}
async fn compose_mail(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<ComposeMail>,
) -> Result<HttpResponse, ApiError> {
    let subject = i.subject.trim();
    let text = i.text.trim();
    if subject.is_empty() || subject.len() > 40 || text.is_empty() {
        return Err(ApiError::BadRequest("invalid mail".into()));
    }
    let target: Option<i64> = if let Ok(id) = i.to.trim().parse::<i64>() {
        sqlx::query_scalar("SELECT id FROM users WHERE id=$1")
            .bind(id)
            .fetch_optional(&s.pool)
            .await?
    } else {
        sqlx::query_scalar("SELECT id FROM users WHERE login=$1")
            .bind(i.to.trim())
            .fetch_optional(&s.pool)
            .await?
    };
    let target = target.ok_or(ApiError::NotFound)?;
    if target == u.0 {
        return Err(ApiError::BadRequest(
            "you can not send email to yourself".into(),
        ));
    }
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO mails(sender_id,recipient_id,subject,text) VALUES($1,$2,$3,$4) RETURNING id",
    )
    .bind(u.0)
    .bind(target)
    .bind(subject)
    .bind(text)
    .fetch_one(&s.pool)
    .await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"id":id,"to":target})))
}
async fn delete_mail(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    if sqlx::query(
        "UPDATE mails SET is_deleted=true WHERE id=$1 AND recipient_id=$2 AND NOT is_deleted",
    )
    .bind(*id)
    .bind(u.0)
    .execute(&s.pool)
    .await?
    .rows_affected()
        == 0
    {
        return Err(ApiError::NotFound);
    }
    Ok(HttpResponse::NoContent().finish())
}
async fn missions(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let v=rows(&s,u.0,"SELECT id,type,status,hirer,victim,info,info2,prize,level FROM missions WHERE user_id=$1 OR status=1 ORDER BY date_generated DESC").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"type":r.get::<i16,_>("type"),"status":r.get::<i16,_>("status"),"hirer":r.get::<i64,_>("hirer"),"victim":r.get::<i64,_>("victim"),"info":r.get::<i64,_>("info"),"info2":r.get::<i64,_>("info2"),"prize":r.get::<i64,_>("prize"),"level":r.get::<i16,_>("level")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn mission_command(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
    i: web::Json<MissionCommand>,
) -> Result<HttpResponse, ApiError> {
    if i.action == "accept" {
        let active: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM missions WHERE user_id=$1 AND status=2")
                .bind(u.0)
                .fetch_one(&s.pool)
                .await?;
        if active != 0 {
            return Err(ApiError::BadRequest("you already are in a mission".into()));
        }
        let level: Option<i16> =
            sqlx::query_scalar("SELECT level FROM missions WHERE id=$1 AND status=1")
                .bind(*id)
                .fetch_optional(&s.pool)
                .await?;
        let level = level.ok_or(ApiError::NotFound)?;
        let version: Option<i32> = sqlx::query_scalar("SELECT MAX(soft_version) FROM software WHERE user_id=$1 AND NOT is_npc AND soft_type=1")
            .bind(u.0).fetch_one(&s.pool).await?;
        let player_level = match version.unwrap_or(0) {
            60.. => 3,
            30.. => 2,
            _ => 1,
        };
        if level != player_level {
            return Err(ApiError::BadRequest(
                "mission level does not match your best cracker".into(),
            ));
        }
    }
    let n =
        match i.action.as_str() {
            "accept" => {
                sqlx::query("UPDATE missions SET status=2,user_id=$1 WHERE id=$2 AND status=1")
                    .bind(u.0)
                    .bind(*id)
                    .execute(&s.pool)
                    .await?
            }
            "abort" => sqlx::query(
                "UPDATE missions SET status=4 WHERE id=$1 AND user_id=$2 AND status=2 AND type<=49",
            )
            .bind(*id)
            .bind(u.0)
            .execute(&s.pool)
            .await?,
            _ => return Err(ApiError::BadRequest("unsupported mission action".into())),
        };
    if n.rows_affected() == 0 {
        return Err(ApiError::BadRequest("mission action not allowed".into()));
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"id":*id,"action":i.action})))
}
async fn bank_accounts(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let v=rows(&s,u.0,"SELECT id,bank_acc,bank_id,cash FROM bank_accounts WHERE bank_user=$1 ORDER BY date_created DESC").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"account":r.get::<i64,_>("bank_acc"),"bank_id":r.get::<i64,_>("bank_id"),"cash":r.get::<i64,_>("cash")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn create_bank_account(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<CreateBankAccount>,
) -> Result<HttpResponse, ApiError> {
    if !(1..=9999).contains(&i.bank_id)
        || i.password.len() != 6
        || !i.password.bytes().all(|b| b.is_ascii_alphanumeric())
    {
        return Err(ApiError::BadRequest("invalid bank account data".into()));
    }
    let existing: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_accounts WHERE bank_user=$1 AND bank_id=$2")
            .bind(u.0)
            .bind(i.bank_id)
            .fetch_one(&s.pool)
            .await?;
    if existing > 0 {
        return Err(ApiError::BadRequest(
            "account already exists for this bank".into(),
        ));
    }
    for _ in 0..5 {
        let account = rand::rng().random_range(111_111_111_i64..=999_999_999);
        let result=sqlx::query("INSERT INTO bank_accounts(bank_acc,bank_pass,bank_id,bank_user) VALUES($1,$2,$3,$4) ON CONFLICT(bank_acc) DO NOTHING").bind(account).bind(&i.password).bind(i.bank_id).bind(u.0).execute(&s.pool).await?;
        if result.rows_affected() == 1 {
            sqlx::query("INSERT INTO log(user_id,text) VALUES($1,$2)")
                .bind(u.0)
                .bind(format!("Bank account {account} opened"))
                .execute(&s.pool)
                .await?;
            return Ok(HttpResponse::Created()
                .json(serde_json::json!({"account":account,"bank_id":i.bank_id,"cash":0})));
        }
    }
    Err(ApiError::Internal(sqlx::Error::RowNotFound))
}
async fn bank_transfer(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<BankTransfer>,
) -> Result<HttpResponse, ApiError> {
    if i.amount <= 0 || i.from_account == i.to_account {
        return Err(ApiError::BadRequest("invalid transfer".into()));
    }
    let mut tx = s.pool.begin().await?;
    let a = sqlx::query(
        "SELECT id,bank_pass,bank_user,cash FROM bank_accounts WHERE bank_acc=$1 FOR UPDATE",
    )
    .bind(i.from_account)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;
    let b = sqlx::query("SELECT id,bank_user FROM bank_accounts WHERE bank_acc=$1 FOR UPDATE")
        .bind(i.to_account)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError::NotFound)?;
    if a.get::<i64, _>("bank_user") != u.0 || a.get::<String, _>("bank_pass") != i.password {
        return Err(ApiError::Unauthorized);
    }
    if a.get::<i64, _>("cash") < i.amount {
        return Err(ApiError::BadRequest("insufficient funds".into()));
    }
    sqlx::query("UPDATE bank_accounts SET cash=cash-$1 WHERE id=$2")
        .bind(i.amount)
        .bind(a.get::<i64, _>("id"))
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE bank_accounts SET cash=cash+$1 WHERE id=$2")
        .bind(i.amount)
        .bind(b.get::<i64, _>("id"))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok()
        .json(serde_json::json!({"from":i.from_account,"to":i.to_account,"amount":i.amount})))
}
async fn ranking(s: web::Data<AppState>, _: CurrentUser) -> Result<HttpResponse, ApiError> {
    let v=sqlx::query("SELECT r.rank,u.id,u.login FROM ranking_user r JOIN users u ON u.id=r.user_id ORDER BY r.rank LIMIT 100").fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"rank":r.get::<i32,_>("rank"),"user_id":r.get::<i64,_>("id"),"login":r.get::<String,_>("login")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn friends(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let v=rows(&s,u.0,"SELECT u.id,u.login FROM users_friends f JOIN users u ON u.id=CASE WHEN f.user_id=$1 THEN f.friend_id ELSE f.user_id END WHERE f.user_id=$1 OR f.friend_id=$1").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"login":r.get::<String,_>("login")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn request_friend(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<FriendRequest>,
) -> Result<HttpResponse, ApiError> {
    if i.user_id == u.0 {
        return Err(ApiError::BadRequest("you can not add yourself".into()));
    }
    let id:Option<i64>=sqlx::query_scalar("INSERT INTO friend_requests(user_id,requested_by) SELECT $1,$2 WHERE EXISTS(SELECT 1 FROM users WHERE id=$1) ON CONFLICT DO NOTHING RETURNING id").bind(i.user_id).bind(u.0).fetch_optional(&s.pool).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"id":id.ok_or(ApiError::BadRequest("request already exists or user missing".into()))?})))
}
async fn accept_friend(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let mut tx = s.pool.begin().await?;
    let friend: Option<i64> = sqlx::query_scalar(
        "DELETE FROM friend_requests WHERE id=$1 AND user_id=$2 RETURNING requested_by",
    )
    .bind(*id)
    .bind(u.0)
    .fetch_optional(&mut *tx)
    .await?;
    let friend = friend.ok_or(ApiError::NotFound)?;
    sqlx::query(
        "INSERT INTO users_friends(user_id,friend_id) VALUES($1,$2),($2,$1) ON CONFLICT DO NOTHING",
    )
    .bind(u.0)
    .bind(friend)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"friend_id":friend})))
}
async fn player_log(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT id,text FROM log WHERE user_id=$1 ORDER BY id DESC LIMIT 200")
        .bind(u.0)
        .fetch_all(&s.pool)
        .await?
        .into_iter()
        .map(|r| serde_json::json!({"id":r.get::<i64,_>("id"),"text":r.get::<String,_>("text")}))
        .collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(rows))
}
async fn news(s: web::Data<AppState>, _: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows=sqlx::query("SELECT id,author,title,content,published_at,type,info1,info2 FROM news ORDER BY published_at DESC LIMIT 50").fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"author":r.get::<String,_>("author"),"title":r.get::<String,_>("title"),"content":r.get::<String,_>("content"),"published_at":r.get::<chrono::DateTime<chrono::Utc>,_>("published_at"),"type":r.get::<i16,_>("type")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(rows))
}
async fn internet_targets(
    s: web::Data<AppState>,
    _: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let rows=sqlx::query("SELECT n.id,n.npc_ip,n.npc_type,n.down_until,i.name,i.web FROM npc n LEFT JOIN npc_info i ON i.npc_id=n.id WHERE n.down_until<=now() ORDER BY n.npc_type,n.npc_ip LIMIT 200").fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"ip":r.get::<i64,_>("npc_ip"),"type":r.get::<i16,_>("npc_type"),"name":r.get::<Option<String>,_>("name"),"web":r.get::<Option<String>,_>("web")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(rows))
}
// ---- Internet: port of classes/Internet.class.php ----
// The legacy resolved an IP by looking at npc first and only then at users
// (Player::getIDByIP). That order is what decides whether a target is an NPC or a player,
// so it is kept: an NPC IP wins over a player holding the same one.
struct Remote {
    is_npc: bool,
    id: i64,
    name: String,
    npc_type: i16,
    down: bool,
}
async fn resolve_ip(s: &AppState, ip: i64) -> Result<Remote, ApiError> {
    if let Some(r)=sqlx::query("SELECT n.id,n.npc_type,n.down_until>now() AS down,COALESCE(i.name,'') AS name FROM npc n LEFT JOIN npc_info i ON i.npc_id=n.id WHERE n.npc_ip=$1").bind(ip).fetch_optional(&s.pool).await? {
        return Ok(Remote {
            is_npc: true,
            id: r.get("id"),
            name: r.get("name"),
            npc_type: r.get("npc_type"),
            down: r.get("down"),
        });
    }
    let r = sqlx::query("SELECT id,login FROM users WHERE game_ip=$1")
        .bind(ip)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Remote {
        is_npc: false,
        id: r.get("id"),
        name: r.get("login"),
        npc_type: 0,
        down: false,
    })
}
// Port of Internet::navigate. Returns what the legacy painted in showPageInfo and the
// sidebar: who sits on that IP, whether it is down, whether access is already granted and
// whether it is bookmarked.
async fn internet_navigate(
    s: web::Data<AppState>,
    u: CurrentUser,
    ip: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let ip = ip.into_inner();
    let t = resolve_ip(&s, ip).await?;
    // history_add kept five IPs in the 'ip-data' cookie, moving a revisited one to the end.
    sqlx::query("INSERT INTO internet_history(user_id,ip) VALUES($1,$2) ON CONFLICT(user_id,ip) DO UPDATE SET visited_at=now()").bind(u.0).bind(ip).execute(&s.pool).await?;
    let access:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM internet_access WHERE user_id=$1 AND ip=$2)").bind(u.0).bind(ip).fetch_one(&s.pool).await?;
    let bookmarked:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM internet_important WHERE user_id=$1 AND ip=$2)").bind(u.0).bind(ip).fetch_one(&s.pool).await?;
    // A finished PORT_SCAN leaves a row here; the legacy read the same fact off the session.
    let scanned:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM internet_scan WHERE user_id=$1 AND ip=$2)").bind(u.0).bind(ip).fetch_one(&s.pool).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"ip":ip,"kind":if t.is_npc {"npc"} else {"player"},"id":t.id,"name":t.name,"type":t.npc_type,"down":t.down,"access":access,"bookmarked":bookmarked,"scanned":scanned})))
}
// Port of Internet::verifyLogin, the 'root' branch. The legacy 'download' and 'clan' users
// depend on type 8 and type 10 NPCs, which are not ported yet.
async fn internet_login(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<InternetLogin>,
) -> Result<HttpResponse, ApiError> {
    if i.user.trim() != "root" {
        return Err(ApiError::BadRequest("UNKNOWN_USER".into()));
    }
    let t = resolve_ip(&s, i.ip).await?;
    if t.down {
        return Err(ApiError::BadRequest("SERVER_DOWN".into()));
    }
    let expected: String = if t.is_npc {
        sqlx::query_scalar("SELECT npc_pass FROM npc WHERE id=$1")
            .bind(t.id)
            .fetch_one(&s.pool)
            .await?
    } else {
        sqlx::query_scalar("SELECT game_pass FROM users WHERE id=$1")
            .bind(t.id)
            .fetch_one(&s.pool)
            .await?
    };
    if i.password != expected {
        return Err(ApiError::BadRequest("BAD_PASSWORD".into()));
    }
    sqlx::query("INSERT INTO internet_access(user_id,ip,via_exploit) VALUES($1,$2,false) ON CONFLICT(user_id,ip) DO UPDATE SET granted_at=now()").bind(u.0).bind(i.ip).execute(&s.pool).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"access":true,"name":t.name})))
}
// Port of both branches of Internet::hack. 'bf' pits the attacker cracker against the
// victim hasher and demands certification 2; 'scan' pits the port scanner against the
// firewall and demands 3. When the attacker version falls short the legacy bailed out with
// BAD_CRACKER.
async fn internet_hack(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<InternetHack>,
) -> Result<HttpResponse, ApiError> {
    let (mine, theirs, needed, action): (i16, i16, i16, i16) = match i.kind.as_str() {
        "bf" => (1, 2, 2, 11),
        "scan" => (3, 4, 3, 15),
        _ => return Err(ApiError::BadRequest("unsupported hack type".into())),
    };
    let level:i16=sqlx::query_scalar("SELECT cert_level FROM certifications WHERE user_id=$1").bind(u.0).fetch_optional(&s.pool).await?.unwrap_or(0);
    if level < needed {
        return Err(ApiError::BadRequest("NO_CERTIFICATION".into()));
    }
    let t = resolve_ip(&s, i.ip).await?;
    if t.down {
        return Err(ApiError::BadRequest("SERVER_DOWN".into()));
    }
    let attacker:Option<i32>=sqlx::query_scalar("SELECT max(soft_version) FROM software WHERE user_id=$1 AND soft_type=$2 AND NOT soft_hidden AND NOT is_npc").bind(u.0).bind(mine).fetch_one(&s.pool).await?;
    let Some(attacker) = attacker else {
        return Err(ApiError::BadRequest("NO_SOFT".into()));
    };
    // getBestSoftware read the victim's defences straight out of software, with isNPC
    // picking the id space. 0011 dropped the users(id) constraint that made that
    // impossible for NPCs, so one query now covers both kinds of target.
    let defender:i32=sqlx::query_scalar("SELECT COALESCE(max(soft_version),0) FROM software WHERE user_id=$1 AND soft_type=$2 AND NOT soft_hidden AND is_npc=$3").bind(t.id).bind(theirs).bind(t.is_npc).fetch_one(&s.pool).await?;
    if attacker < defender {
        return Err(ApiError::BadRequest("BAD_CRACKER".into()));
    }
    let seconds: i32 = 60;
    let dup:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM processes WHERE creator_id=$1 AND action=$2 AND victim_id=$3 AND completed_at IS NULL)").bind(u.0).bind(action).bind(t.id).fetch_one(&s.pool).await?;
    if dup {
        return Err(ApiError::BadRequest("identical process already active".into()));
    }
    let info = if t.is_npc { "npc" } else { "player" };
    let pid:i64=sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,info,info_str,time_end,time_ideal) VALUES($1,$2,$3,$4,$4,now()+($5::integer*interval '1 second'),$5) RETURNING pid").bind(u.0).bind(t.id).bind(action).bind(info).bind(seconds).fetch_one(&s.pool).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"process_id":pid,"action":action,"seconds":seconds,"attacker_version":attacker,"defender_version":defender})))
}
// Port of hardwareItens.php. The legacy priced parts in static arrays and billed the
// upgrade to a bank account, the same way the university charges for a certification.
async fn hardware_shop(_: CurrentUser) -> HttpResponse {
    let v = HARDWARE_PARTS
        .iter()
        .map(|p| {
            serde_json::json!({"kind":p.kind,"id":p.id,"name":p.name,"power":p.power,"price":p.price})
        })
        .collect::<Vec<_>>();
    HttpResponse::Ok().json(v)
}
// Buying a part replaces what the machine had rather than adding to it, so a cheaper part
// is a downgrade and the shop lets you make that mistake, as the legacy did.
async fn buy_hardware(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<BuyPart>,
) -> Result<HttpResponse, ApiError> {
    let p = HARDWARE_PARTS
        .iter()
        .find(|p| p.kind == i.kind && p.id == i.part)
        .ok_or_else(|| ApiError::BadRequest("unknown part".into()))?;
    let mut tx = s.pool.begin().await?;
    let cash: Option<i64> = sqlx::query_scalar(
        "SELECT cash FROM bank_accounts WHERE bank_acc=$1 AND bank_user=$2 FOR UPDATE",
    )
    .bind(i.account)
    .bind(u.0)
    .fetch_optional(&mut *tx)
    .await?;
    if cash.unwrap_or(0) < p.price {
        return Err(ApiError::BadRequest("insufficient funds".into()));
    }
    // A disk cannot shrink below what is already stored on it, nor memory below what is
    // loaded; the legacy had no such guard because it never let you downgrade in practice.
    let floor: i64 = match p.kind {
        "hdd" => sqlx::query_scalar("SELECT COALESCE(sum(soft_size),0)::bigint FROM software WHERE user_id=$1 AND NOT is_npc").bind(u.0).fetch_one(&mut *tx).await?,
        "ram" => sqlx::query_scalar("SELECT COALESCE(sum(s.soft_ram),0)::bigint FROM software s JOIN software_running r ON r.soft_id=s.id WHERE r.user_id=$1 AND NOT r.is_npc").bind(u.0).fetch_one(&mut *tx).await?,
        _ => 0,
    };
    if (p.power as i64) < floor {
        return Err(ApiError::BadRequest("PART_TOO_SMALL".into()));
    }
    sqlx::query("UPDATE bank_accounts SET cash=cash-$1 WHERE bank_acc=$2 AND bank_user=$3")
        .bind(p.price)
        .bind(i.account)
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    let sql = match p.kind {
        "cpu" => "UPDATE hardware SET cpu=$1 WHERE user_id=$2 AND NOT is_npc",
        "ram" => "UPDATE hardware SET ram=$1 WHERE user_id=$2 AND NOT is_npc",
        "hdd" => "UPDATE hardware SET hdd=$1 WHERE user_id=$2 AND NOT is_npc",
        _ => "UPDATE hardware SET net=$1 WHERE user_id=$2 AND NOT is_npc",
    };
    sqlx::query(sql)
        .bind(p.power as f32)
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE users_stats SET money_hardware=money_hardware+$1 WHERE uid=$2")
        .bind(p.price)
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"kind":p.kind,"name":p.name,"power":p.power,"paid":p.price})))
}
// Port of createsoft.php. Writing your own program is the other way to get one, next to
// stealing it, and it is what makes an empty disk useful.
async fn create_software(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<NewSoftware>,
) -> Result<HttpResponse, ApiError> {
    let name = i.name.trim();
    if name.is_empty() || name.len() > 25 {
        return Err(ApiError::BadRequest("invalid software name".into()));
    }
    if !SOFTWARE_TYPES.iter().any(|(_, id)| *id as i16 == i.soft_type) {
        return Err(ApiError::BadRequest("unknown software type".into()));
    }
    if i.size <= 0 || i.ram < 0 {
        return Err(ApiError::BadRequest("invalid size".into()));
    }
    let total:Option<f32>=sqlx::query_scalar("SELECT sum(hdd)::real FROM hardware WHERE user_id=$1 AND NOT is_npc").bind(u.0).fetch_one(&s.pool).await?;
    let used:Option<i64>=sqlx::query_scalar("SELECT sum(soft_size)::bigint FROM software WHERE user_id=$1 AND NOT is_npc").bind(u.0).fetch_one(&s.pool).await?;
    if (total.unwrap_or(0.0) as i64) < used.unwrap_or(0) + i.size as i64 {
        return Err(ApiError::BadRequest("NO_DISK_SPACE".into()));
    }
    let id:i64=sqlx::query_scalar("INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type) VALUES($1,$2,1,$3,$4,$5) RETURNING id").bind(u.0).bind(name).bind(i.size).bind(i.ram).bind(i.soft_type).fetch_one(&s.pool).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"id":id,"name":name,"version":1})))
}
// Port of the file list Internet::showPageInfo drew once you were inside a machine.
async fn remote_software(
    s: web::Data<AppState>,
    u: CurrentUser,
    ip: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let ip = ip.into_inner();
    let t = resolve_ip(&s, ip).await?;
    require_access(&s, u.0, ip).await?;
    let v=sqlx::query("SELECT id,soft_name,soft_version,soft_size,soft_ram,soft_type,is_folder FROM software WHERE user_id=$1 AND is_npc=$2 AND NOT soft_hidden ORDER BY soft_name").bind(t.id).bind(t.is_npc).fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"name":r.get::<String,_>("soft_name"),"version":r.get::<i32,_>("soft_version"),"size":r.get::<i32,_>("soft_size"),"ram":r.get::<i32,_>("soft_ram"),"type":r.get::<i16,_>("soft_type"),"folder":r.get::<bool,_>("is_folder")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
/// A bitcoin wallet, a folder and anything from 90 up are the three things case 1 of
/// completeProcess refused to copy. Checked when the transfer is asked for rather than
/// only when it lands, so the player is not left waiting on a process that cannot work.
fn transferable(soft_type: i16) -> bool {
    soft_type != 19 && soft_type != 31 && soft_type < 90
}
// Port of the DOWNLOAD and UPLOAD branches of Internet::handlePost. Both open a process;
// the copy itself happens in completeProcess, which is where the legacy checked disk space
// and refused duplicates.
async fn transfer_software(
    s: web::Data<AppState>,
    u: CurrentUser,
    ip: web::Path<i64>,
    i: web::Json<Transfer>,
) -> Result<HttpResponse, ApiError> {
    let ip = ip.into_inner();
    let t = resolve_ip(&s, ip).await?;
    require_access(&s, u.0, ip).await?;
    let action: i16 = match i.direction.as_str() {
        "download" => 1,
        "upload" => 2,
        _ => return Err(ApiError::BadRequest("unsupported transfer".into())),
    };
    // A download reads from the remote machine, an upload from the player's own.
    let (owner, owner_is_npc) = if action == 1 {
        (t.id, t.is_npc)
    } else {
        (u.0, false)
    };
    let row=sqlx::query("SELECT soft_type,soft_name,soft_size FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3 AND NOT soft_hidden").bind(i.software_id).bind(owner).bind(owner_is_npc).fetch_optional(&s.pool).await?.ok_or(ApiError::NotFound)?;
    if !transferable(row.get::<i16, _>("soft_type")) {
        return Err(ApiError::BadRequest("CANNOT_TRANSFER".into()));
    }
    let seconds: i32 = 30;
    let dup:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM processes WHERE creator_id=$1 AND action=$2 AND victim_id=$3 AND soft_id=$4 AND completed_at IS NULL)").bind(u.0).bind(action).bind(t.id).bind(i.software_id).fetch_one(&s.pool).await?;
    if dup {
        return Err(ApiError::BadRequest("identical process already active".into()));
    }
    let info = if t.is_npc { "npc" } else { "player" };
    let pid:i64=sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal) VALUES($1,$2,$3,$4,$5,$5,now()+($6::integer*interval '1 second'),$6) RETURNING pid").bind(u.0).bind(t.id).bind(action).bind(i.software_id).bind(info).bind(seconds).fetch_one(&s.pool).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"process_id":pid,"action":action,"seconds":seconds})))
}
// Port of the DELETE branch. Deleting a file on a machine you broke into is how the legacy
// let you take something away from a victim rather than just copy it.
async fn delete_remote_software(
    s: web::Data<AppState>,
    u: CurrentUser,
    path: web::Path<(i64, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (ip, soft) = path.into_inner();
    let t = resolve_ip(&s, ip).await?;
    require_access(&s, u.0, ip).await?;
    let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3)").bind(soft).bind(t.id).bind(t.is_npc).fetch_one(&s.pool).await?;
    if !exists {
        return Err(ApiError::NotFound);
    }
    let seconds: i32 = 30;
    let info = if t.is_npc { "npc" } else { "player" };
    let pid:i64=sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal) VALUES($1,$2,3,$3,$4,$4,now()+($5::integer*interval '1 second'),$5) RETURNING pid").bind(u.0).bind(t.id).bind(soft).bind(info).bind(seconds).fetch_one(&s.pool).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"process_id":pid,"seconds":seconds})))
}
// Port of log.php and logEdit.php against a remote machine. The legacy gated both on
// $_SESSION['HACKED'] holding the current IP; here the same gate is the internet_access
// row a finished crack or a correct login leaves behind.
async fn remote_log(
    s: web::Data<AppState>,
    u: CurrentUser,
    ip: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let ip = ip.into_inner();
    let t = resolve_ip(&s, ip).await?;
    require_access(&s, u.0, ip).await?;
    // getLogValue($uid, $npc): one row per machine, newest entry on top.
    let text:Option<String>=sqlx::query_scalar("SELECT text FROM log WHERE user_id=$1 AND is_npc=$2").bind(t.id).bind(t.is_npc).fetch_optional(&s.pool).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"ip":ip,"text":text.unwrap_or_default()})))
}
// Port of logEdit.php. The legacy never rewrote the log inline: it parked the new text in
// log_edit through tmpLog and opened an E_LOG process, so covering your tracks takes time
// and can be caught half-done. The worker applies it when the process finishes.
async fn edit_remote_log(
    s: web::Data<AppState>,
    u: CurrentUser,
    ip: web::Path<i64>,
    i: web::Json<LogEdit>,
) -> Result<HttpResponse, ApiError> {
    let ip = ip.into_inner();
    let t = resolve_ip(&s, ip).await?;
    require_access(&s, u.0, ip).await?;
    let seconds: i32 = 30;
    let pid:i64=sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,info,info_str,time_end,time_ideal) VALUES($1,$2,8,$3,$3,now()+($4::integer*interval '1 second'),$4) RETURNING pid").bind(u.0).bind(t.id).bind(if t.is_npc {"npc"} else {"player"}).bind(seconds).fetch_one(&s.pool).await?;
    sqlx::query("INSERT INTO log_edit(vic_id,is_npc,editor_id,log_text,process_id) VALUES($1,$2,$3,$4,$5)")
        .bind(t.id)
        .bind(t.is_npc)
        .bind(u.0)
        .bind(&i.text)
        .bind(pid)
        .execute(&s.pool)
        .await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"process_id":pid,"seconds":seconds})))
}
// $_SESSION['HACKED'] == $ip, the check every remote screen in the legacy started with.
async fn require_access(s: &AppState, user: i64, ip: i64) -> Result<(), ApiError> {
    let ok:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM internet_access WHERE user_id=$1 AND ip=$2)").bind(user).bind(ip).fetch_one(&s.pool).await?;
    if ok {
        return Ok(());
    }
    Err(ApiError::BadRequest("NOT_LOGGED_IN".into()))
}
// Port of important_show. The legacy joined npc_info to give every bookmark a name.
async fn internet_bookmarks(
    s: web::Data<AppState>,
    u: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let v=sqlx::query("SELECT b.ip,COALESCE(i.name,us.login,'') AS name FROM internet_important b LEFT JOIN npc n ON n.npc_ip=b.ip LEFT JOIN npc_info i ON i.npc_id=n.id LEFT JOIN users us ON us.game_ip=b.ip WHERE b.user_id=$1 ORDER BY b.id").bind(u.0).fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"ip":r.get::<i64,_>("ip"),"name":r.get::<String,_>("name")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn add_bookmark(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<IpBody>,
) -> Result<HttpResponse, ApiError> {
    resolve_ip(&s, i.ip).await?;
    sqlx::query("INSERT INTO internet_important(user_id,ip) VALUES($1,$2) ON CONFLICT DO NOTHING")
        .bind(u.0)
        .bind(i.ip)
        .execute(&s.pool)
        .await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"ip":i.ip})))
}
async fn remove_bookmark(
    s: web::Data<AppState>,
    u: CurrentUser,
    ip: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    if sqlx::query("DELETE FROM internet_important WHERE user_id=$1 AND ip=$2")
        .bind(u.0)
        .bind(*ip)
        .execute(&s.pool)
        .await?
        .rows_affected()
        == 0
    {
        return Err(ApiError::NotFound);
    }
    Ok(HttpResponse::NoContent().finish())
}
// Port of history_show. The legacy kept five entries in a cookie; the trimming happens on
// read so the full history survives in the database.
async fn internet_history(
    s: web::Data<AppState>,
    u: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let v=sqlx::query("SELECT h.ip,h.visited_at,COALESCE(i.name,us.login,'') AS name FROM internet_history h LEFT JOIN npc n ON n.npc_ip=h.ip LEFT JOIN npc_info i ON i.npc_id=n.id LEFT JOIN users us ON us.game_ip=h.ip WHERE h.user_id=$1 ORDER BY h.visited_at DESC LIMIT 5").bind(u.0).fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"ip":r.get::<i64,_>("ip"),"name":r.get::<String,_>("name"),"visited_at":r.get::<chrono::DateTime<chrono::Utc>,_>("visited_at")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
}
async fn server_stats(s: web::Data<AppState>, _: CurrentUser) -> Result<HttpResponse, ApiError> {
    let payload: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT payload FROM system_stats WHERE key='server'")
            .fetch_optional(&s.pool)
            .await?;
    Ok(HttpResponse::Ok().json(payload.unwrap_or_else(|| serde_json::json!({}))))
}
async fn settings(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let r = sqlx::query("SELECT login,email,premium,last_login FROM users WHERE id=$1")
        .bind(u.0)
        .fetch_one(&s.pool)
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"login":r.get::<String,_>("login"),"email":r.get::<String,_>("email"),"premium":r.get::<bool,_>("premium"),"last_login":r.get::<chrono::DateTime<chrono::Utc>,_>("last_login")})))
}
async fn save_settings(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<SettingsUpdate>,
) -> Result<HttpResponse, ApiError> {
    if i.current_password.is_empty() {
        return Err(ApiError::BadRequest("current password is required".into()));
    }
    let mut tx = s.pool.begin().await?;
    let current: String = sqlx::query_scalar("SELECT password FROM users WHERE id=$1 FOR UPDATE")
        .bind(u.0)
        .fetch_one(&mut *tx)
        .await?;
    if !verify(&i.current_password, &current).map_err(|_| ApiError::Unauthorized)? {
        return Err(ApiError::Unauthorized);
    }
    if let Some(email) = &i.email {
        let email = email.trim();
        if !email.contains('@') || email.len() > 50 {
            return Err(ApiError::BadRequest("invalid email".into()));
        }
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email=$1 AND id<>$2)")
                .bind(email)
                .bind(u.0)
                .fetch_one(&mut *tx)
                .await?;
        if exists {
            return Err(ApiError::BadRequest("email already exists".into()));
        }
        sqlx::query("UPDATE users SET email=$1 WHERE id=$2")
            .bind(email)
            .bind(u.0)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(password) = &i.new_password {
        if password.len() < 8 {
            return Err(ApiError::BadRequest(
                "new password must have 8 characters".into(),
            ));
        }
        let password =
            hash(password, 13).map_err(|_| ApiError::BadRequest("cannot hash password".into()))?;
        sqlx::query("UPDATE users SET password=$1 WHERE id=$2")
            .bind(password)
            .bind(u.0)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT INTO log(user_id,text) VALUES($1,'Account settings updated')")
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"saved":true})))
}
fn certification(level: i16) -> Option<(&'static str, &'static str, i64)> {
    match level {
        1 => Some((
            "Basic tutorial",
            "Follow this tutorial to learn how to play",
            0,
        )),
        2 => Some(("Hacking 101", "Learn the basics of hacking", 0)),
        3 => Some((
            "Intermediate hacking",
            "Improve your hacking techniques",
            50,
        )),
        4 => Some(("Advanced hacking", "Meet the wonders of DDoS world", 200)),
        5 => Some(("DDoS security", "Learn to protect yourself from DDoS", 500)),
        _ => None,
    }
}
async fn certifications(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let level: i16 = sqlx::query_scalar("SELECT cert_level FROM certifications WHERE user_id=$1")
        .bind(u.0)
        .fetch_one(&s.pool)
        .await?;
    let learning: bool = sqlx::query_scalar("SELECT learning FROM users_learning WHERE user_id=$1")
        .bind(u.0)
        .fetch_one(&s.pool)
        .await?;
    let current: Option<i64> =
        sqlx::query_scalar("SELECT puzzle_id FROM users_puzzle WHERE user_id=$1")
            .bind(u.0)
            .fetch_optional(&s.pool)
            .await?;
    let items=(1..=5).filter_map(|n|certification(n).map(|(name,description,price)|serde_json::json!({"level":n,"name":name,"description":description,"price":price,"completed":level>=n,"available":learning&&level==n-1,"learning":!learning&&current==Some(n as i64)}))).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(serde_json::json!({"level":level,"items":items})))
}
async fn start_certification(
    s: web::Data<AppState>,
    u: CurrentUser,
    level: web::Path<i16>,
    i: web::Json<CertificationStart>,
) -> Result<HttpResponse, ApiError> {
    let (_, _, price) = certification(*level)
        .ok_or_else(|| ApiError::BadRequest("invalid certification".into()))?;
    let mut tx = s.pool.begin().await?;
    let current: i16 =
        sqlx::query_scalar("SELECT cert_level FROM certifications WHERE user_id=$1 FOR UPDATE")
            .bind(u.0)
            .fetch_one(&mut *tx)
            .await?;
    let learning: bool =
        sqlx::query_scalar("SELECT learning FROM users_learning WHERE user_id=$1 FOR UPDATE")
            .bind(u.0)
            .fetch_one(&mut *tx)
            .await?;
    if current != *level - 1 || !learning {
        return Err(ApiError::BadRequest(
            "certification is not available".into(),
        ));
    }
    if price > 0 {
        let account = i
            .account
            .ok_or_else(|| ApiError::BadRequest("bank account required".into()))?;
        let cash: Option<i64> = sqlx::query_scalar(
            "SELECT cash FROM bank_accounts WHERE bank_acc=$1 AND bank_user=$2 FOR UPDATE",
        )
        .bind(account)
        .bind(u.0)
        .fetch_optional(&mut *tx)
        .await?;
        if cash.unwrap_or(0) < price {
            return Err(ApiError::BadRequest("insufficient funds".into()));
        }
        sqlx::query("UPDATE bank_accounts SET cash=cash-$1 WHERE bank_acc=$2 AND bank_user=$3")
            .bind(price)
            .bind(account)
            .bind(u.0)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE users_learning SET learning=false WHERE user_id=$1")
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE users_puzzle SET puzzle_id=$1 WHERE user_id=$2")
        .bind(*level as i64)
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"level":*level,"started":true})))
}
async fn complete_certification(
    s: web::Data<AppState>,
    u: CurrentUser,
    level: web::Path<i16>,
) -> Result<HttpResponse, ApiError> {
    certification(*level).ok_or_else(|| ApiError::BadRequest("invalid certification".into()))?;
    let mut tx = s.pool.begin().await?;
    let active: Option<i64> =
        sqlx::query_scalar("SELECT puzzle_id FROM users_puzzle WHERE user_id=$1 FOR UPDATE")
            .bind(u.0)
            .fetch_optional(&mut *tx)
            .await?;
    if active != Some(*level as i64) {
        return Err(ApiError::BadRequest(
            "certification is not being learned".into(),
        ));
    }
    sqlx::query("UPDATE certifications SET cert_level=$1 WHERE user_id=$2")
        .bind(*level)
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE users_learning SET learning=true WHERE user_id=$1")
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE users_puzzle SET puzzle_id=0 WHERE user_id=$1")
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"level":*level,"completed":true})))
}
async fn profile(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let r=sqlx::query("SELECT u.id,u.login,u.game_ip,p.biography,p.visible,COALESCE(c.reputation,0) AS reputation,COALESCE(rk.rank,-1) AS rank,st.mission_count,st.hack_count,st.money_earned FROM users u JOIN player_profiles p ON p.user_id=u.id LEFT JOIN cache c ON c.user_id=u.id LEFT JOIN ranking_user rk ON rk.user_id=u.id LEFT JOIN users_stats st ON st.uid=u.id WHERE u.id=$1").bind(u.0).fetch_one(&s.pool).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"id":r.get::<i64,_>("id"),"login":r.get::<String,_>("login"),"game_ip":r.get::<i64,_>("game_ip"),"biography":r.get::<String,_>("biography"),"visible":r.get::<bool,_>("visible"),"reputation":r.get::<i64,_>("reputation"),"rank":r.get::<i32,_>("rank"),"missions":r.get::<i32,_>("mission_count"),"hacks":r.get::<i32,_>("hack_count"),"money_earned":r.get::<i64,_>("money_earned")})))
}
async fn save_profile(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<ProfileUpdate>,
) -> Result<HttpResponse, ApiError> {
    let bio = i.biography.trim();
    if bio.len() > 3000 {
        return Err(ApiError::BadRequest("biography is too long".into()));
    }
    sqlx::query("INSERT INTO player_profiles(user_id,biography,visible) VALUES($1,$2,$3) ON CONFLICT(user_id) DO UPDATE SET biography=EXCLUDED.biography,visible=EXCLUDED.visible,updated_at=now()").bind(u.0).bind(bio).bind(i.visible).execute(&s.pool).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"saved":true})))
}
async fn clans(s: web::Data<AppState>, _: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows=sqlx::query("SELECT c.id,c.clan_ip,c.name,c.nick,c.description,c.slots_total,c.money,c.power,COUNT(m.user_id) AS members FROM clans c LEFT JOIN clan_members m ON m.clan_id=c.id GROUP BY c.id ORDER BY c.power DESC,c.name").fetch_all(&s.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"ip":r.get::<i64,_>("clan_ip"),"name":r.get::<String,_>("name"),"nick":r.get::<String,_>("nick"),"description":r.get::<String,_>("description"),"slots":r.get::<i16,_>("slots_total"),"members":r.get::<i64,_>("members"),"money":r.get::<i64,_>("money"),"power":r.get::<i64,_>("power")})).collect::<Vec<_>>()))
}
async fn create_clan(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<ClanCreate>,
) -> Result<HttpResponse, ApiError> {
    let name = i.name.trim();
    let nick = i.nick.trim().to_uppercase();
    if name.is_empty()
        || name.len() > 25
        || nick.len() != 3
        || !nick.bytes().all(|c| c.is_ascii_alphanumeric())
        || i.description.len() > 3000
    {
        return Err(ApiError::BadRequest("invalid clan data".into()));
    }
    let mut tx = s.pool.begin().await?;
    let already: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clan_members WHERE user_id=$1)")
            .bind(u.0)
            .fetch_one(&mut *tx)
            .await?;
    if already {
        return Err(ApiError::BadRequest("already in a clan".into()));
    }
    let ip = rand::rng().random_range(1_i64..=u32::MAX as i64);
    let id:i64=sqlx::query_scalar("INSERT INTO clans(clan_ip,name,nick,description,created_by) VALUES($1,$2,$3,$4,$5) RETURNING id").bind(ip).bind(name).bind(nick).bind(i.description.trim()).bind(u.0).fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO clan_members(clan_id,user_id,auth_level,hierarchy) VALUES($1,$2,3,0)")
        .bind(id)
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"id":id})))
}
async fn my_clan(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let clan=sqlx::query("SELECT c.id,c.name,c.nick,c.description,c.slots_total,c.money,c.power,m.auth_level FROM clan_members m JOIN clans c ON c.id=m.clan_id WHERE m.user_id=$1").bind(u.0).fetch_optional(&s.pool).await?;
    let Some(c) = clan else {
        return Ok(HttpResponse::Ok().json(serde_json::json!(null)));
    };
    let id: i64 = c.get("id");
    let members=sqlx::query("SELECT u.id,u.login,m.auth_level,m.member_since FROM clan_members m JOIN users u ON u.id=m.user_id WHERE m.clan_id=$1 ORDER BY m.auth_level DESC,m.member_since").bind(id).fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"login":r.get::<String,_>("login"),"auth_level":r.get::<i16,_>("auth_level")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(serde_json::json!({"id":id,"name":c.get::<String,_>("name"),"nick":c.get::<String,_>("nick"),"description":c.get::<String,_>("description"),"slots":c.get::<i16,_>("slots_total"),"money":c.get::<i64,_>("money"),"power":c.get::<i64,_>("power"),"auth_level":c.get::<i16,_>("auth_level"),"members":members})))
}
async fn request_clan_join(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
    i: web::Json<ClanJoin>,
) -> Result<HttpResponse, ApiError> {
    if i.message.len() > 3000 {
        return Err(ApiError::BadRequest("request message is too long".into()));
    }
    let own: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clan_members WHERE user_id=$1)")
            .bind(u.0)
            .fetch_one(&s.pool)
            .await?;
    if own {
        return Err(ApiError::BadRequest("already in a clan".into()));
    }
    let req:Option<i64>=sqlx::query_scalar("INSERT INTO clan_requests(clan_id,user_id,message) SELECT $1,$2,$3 WHERE EXISTS(SELECT 1 FROM clans WHERE id=$1) ON CONFLICT DO NOTHING RETURNING id").bind(*id).bind(u.0).bind(i.message.trim()).fetch_optional(&s.pool).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"id":req.ok_or(ApiError::BadRequest("request already exists or clan missing".into()))?})))
}
async fn clan_requests(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT r.id,r.message,r.created_at,applicant.login FROM clan_requests r JOIN clan_members manager ON manager.clan_id=r.clan_id JOIN users applicant ON applicant.id=r.user_id WHERE manager.user_id=$1 AND manager.auth_level>=2 ORDER BY r.created_at")
        .bind(u.0)
        .fetch_all(&s.pool)
        .await?
        .into_iter()
        .map(|r| serde_json::json!({"id":r.get::<i64,_>("id"),"login":r.get::<String,_>("login"),"message":r.get::<String,_>("message"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at")}))
        .collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(rows))
}
async fn accept_clan_request(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let mut tx = s.pool.begin().await?;
    let request=sqlx::query("SELECT r.clan_id,r.user_id,c.slots_total FROM clan_requests r JOIN clans c ON c.id=r.clan_id JOIN clan_members a ON a.clan_id=r.clan_id WHERE r.id=$1 AND a.user_id=$2 AND a.auth_level>=2 FOR UPDATE").bind(*id).bind(u.0).fetch_optional(&mut *tx).await?.ok_or(ApiError::NotFound)?;
    let clan: i64 = request.get("clan_id");
    let applicant: i64 = request.get("user_id");
    let members: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clan_members WHERE clan_id=$1")
        .bind(clan)
        .fetch_one(&mut *tx)
        .await?;
    if members >= request.get::<i16, _>("slots_total") as i64 {
        return Err(ApiError::BadRequest("clan is full".into()));
    }
    sqlx::query("INSERT INTO clan_members(clan_id,user_id) VALUES($1,$2)")
        .bind(clan)
        .bind(applicant)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM clan_requests WHERE id=$1")
        .bind(*id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"clan_id":clan,"user_id":applicant})))
}
async fn research(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows=sqlx::query("SELECT id,software_id,software_type,new_version,software_name,researched_at FROM research_history WHERE user_id=$1 ORDER BY researched_at DESC").bind(u.0).fetch_all(&s.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"software_id":r.get::<i64,_>("software_id"),"type":r.get::<i16,_>("software_type"),"version":r.get::<i32,_>("new_version"),"name":r.get::<String,_>("software_name"),"researched_at":r.get::<chrono::DateTime<chrono::Utc>,_>("researched_at")})).collect::<Vec<_>>()))
}
async fn start_research(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<ResearchStart>,
) -> Result<HttpResponse, ApiError> {
    // Research was action 17 in the legacy, a process like any other. Raising a version
    // instantly and for free made it the one upgrade path that cost nothing, which is why
    // it now opens a process and the worker applies the new version when it lands.
    let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc)").bind(i.software_id).bind(u.0).fetch_one(&s.pool).await?;
    if !exists {
        return Err(ApiError::NotFound);
    }
    let dup:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM processes WHERE creator_id=$1 AND action=17 AND soft_id=$2 AND completed_at IS NULL)").bind(u.0).bind(i.software_id).fetch_one(&s.pool).await?;
    if dup {
        return Err(ApiError::BadRequest(
            "identical process already active".into(),
        ));
    }
    let seconds: i32 = 60;
    let pid:i64=sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal) VALUES($1,0,17,$2,'local','local',now()+($3::integer*interval '1 second'),$3) RETURNING pid").bind(u.0).bind(i.software_id).bind(seconds).fetch_one(&s.pool).await?;
    Ok(HttpResponse::Created()
        .json(serde_json::json!({"process_id":pid,"software_id":i.software_id,"seconds":seconds})))
}
async fn bitcoin_wallets(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let wallets=sqlx::query("SELECT address,amount::text AS amount,created_at FROM bitcoin_wallets WHERE user_id=$1 ORDER BY created_at").bind(u.0).fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"address":r.get::<String,_>("address"),"amount":r.get::<String,_>("amount"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(wallets))
}
async fn create_bitcoin_wallet(
    s: web::Data<AppState>,
    u: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let address: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(34)
        .map(char::from)
        .collect();
    let key: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    sqlx::query("INSERT INTO bitcoin_wallets(address,user_id,private_key) VALUES($1,$2,$3)")
        .bind(&address)
        .bind(u.0)
        .bind(key)
        .execute(&s.pool)
        .await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"address":address,"amount":"0.0000000"})))
}
async fn bitcoin_transfer(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<BitcoinTransfer>,
) -> Result<HttpResponse, ApiError> {
    let amount: i64 = i
        .amount
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite() && *n > 0.0)
        .map(|n| (n * 10_000_000.0).round() as i64)
        .ok_or_else(|| ApiError::BadRequest("invalid bitcoin amount".into()))?;
    let canonical = format!("{}.{:07}", amount / 10_000_000, amount % 10_000_000);
    if i.from == i.to {
        return Err(ApiError::BadRequest("same bitcoin wallet".into()));
    }
    let mut tx = s.pool.begin().await?;
    let source=sqlx::query("SELECT address,user_id,amount::text AS amount FROM bitcoin_wallets WHERE address=$1 FOR UPDATE").bind(&i.from).fetch_optional(&mut *tx).await?.ok_or(ApiError::NotFound)?;
    let _target = sqlx::query("SELECT address FROM bitcoin_wallets WHERE address=$1 FOR UPDATE")
        .bind(&i.to)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError::NotFound)?;
    if source.get::<i64, _>("user_id") != u.0 {
        return Err(ApiError::Unauthorized);
    }
    let available = (source
        .get::<String, _>("amount")
        .parse::<f64>()
        .unwrap_or(0.0)
        * 10_000_000.0)
        .round() as i64;
    if available < amount {
        return Err(ApiError::BadRequest("insufficient bitcoin funds".into()));
    }
    sqlx::query("UPDATE bitcoin_wallets SET amount=amount-CAST($1 AS numeric) WHERE address=$2")
        .bind(&canonical)
        .bind(&i.from)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE bitcoin_wallets SET amount=amount+CAST($1 AS numeric) WHERE address=$2")
        .bind(&canonical)
        .bind(&i.to)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"from":i.from,"to":i.to,"amount":canonical})))
}
async fn compromised_lists(
    s: web::Data<AppState>,
    u: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let entries=sqlx::query("SELECT id,ip,login,hacked_at,virus_id FROM compromised_lists WHERE user_id=$1 ORDER BY hacked_at DESC").bind(u.0).fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"ip":r.get::<i64,_>("ip"),"login":r.get::<String,_>("login"),"hacked_at":r.get::<chrono::DateTime<chrono::Utc>,_>("hacked_at"),"virus_id":r.get::<i64,_>("virus_id")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(entries))
}
async fn add_compromised_list(
    s: web::Data<AppState>,
    u: CurrentUser,
    i: web::Json<ListEntry>,
) -> Result<HttpResponse, ApiError> {
    let login = i.login.trim();
    if login.is_empty()
        || login.len() > 15
        || i.password.is_empty()
        || i.password.len() > 60
        || i.ip <= 0
    {
        return Err(ApiError::BadRequest("invalid compromised account".into()));
    }
    let id:i64=sqlx::query_scalar("INSERT INTO compromised_lists(user_id,ip,login,password,virus_id) VALUES($1,$2,$3,$4,$5) RETURNING id").bind(u.0).bind(i.ip).bind(login).bind(&i.password).bind(i.virus_id.unwrap_or(0)).fetch_one(&s.pool).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"id":id})))
}
async fn delete_compromised_list(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    if sqlx::query("DELETE FROM compromised_lists WHERE id=$1 AND user_id=$2")
        .bind(*id)
        .bind(u.0)
        .execute(&s.pool)
        .await?
        .rows_affected()
        == 0
    {
        return Err(ApiError::NotFound);
    }
    Ok(HttpResponse::NoContent().finish())
}
async fn wiki_index(s: web::Data<AppState>, _: CurrentUser) -> Result<HttpResponse, ApiError> {
    let pages = sqlx::query("SELECT slug,title,updated_at FROM wiki_pages ORDER BY title")
        .fetch_all(&s.pool).await?
        .into_iter().map(|r| serde_json::json!({"slug":r.get::<String,_>("slug"),"title":r.get::<String,_>("title"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(pages))
}
async fn wiki_page(
    s: web::Data<AppState>,
    _: CurrentUser,
    slug: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let r = sqlx::query("SELECT p.slug,p.title,p.content,p.created_at,p.updated_at,u.login AS author FROM wiki_pages p LEFT JOIN users u ON u.id=p.author_id WHERE p.slug=$1")
        .bind(slug.as_str()).fetch_optional(&s.pool).await?.ok_or(ApiError::NotFound)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"slug":r.get::<String,_>("slug"),"title":r.get::<String,_>("title"),"content":r.get::<String,_>("content"),"author":r.get::<Option<String>,_>("author"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")})))
}
async fn save_wiki_page(
    s: web::Data<AppState>,
    u: CurrentUser,
    slug: web::Path<String>,
    input: web::Json<WikiEdit>,
) -> Result<HttpResponse, ApiError> {
    let slug = slug.into_inner();
    let title = input.title.trim();
    let content = input.content.trim();
    if !valid_slug(&slug) || title.is_empty() || title.len() > 160 || content.is_empty() {
        return Err(ApiError::BadRequest("invalid wiki page".into()));
    }
    let mut tx = s.pool.begin().await?;
    let id: i64 = sqlx::query_scalar("INSERT INTO wiki_pages(slug,title,content,author_id) VALUES($1,$2,$3,$4) ON CONFLICT(slug) DO UPDATE SET title=EXCLUDED.title,content=EXCLUDED.content,author_id=EXCLUDED.author_id,updated_at=now() RETURNING id").bind(&slug).bind(title).bind(content).bind(u.0).fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO wiki_revisions(page_id,author_id,title,content) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(u.0)
        .bind(title)
        .bind(content)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"slug":slug,"id":id})))
}
async fn wiki_revisions(
    s: web::Data<AppState>,
    _: CurrentUser,
    slug: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let revisions=sqlx::query("SELECT r.id,r.title,r.created_at,u.login AS author FROM wiki_revisions r JOIN wiki_pages p ON p.id=r.page_id LEFT JOIN users u ON u.id=r.author_id WHERE p.slug=$1 ORDER BY r.created_at DESC").bind(slug.as_str()).fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"title":r.get::<String,_>("title"),"author":r.get::<Option<String>,_>("author"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(revisions))
}
async fn forum_topics(s: web::Data<AppState>, _: CurrentUser) -> Result<HttpResponse, ApiError> {
    let topics=sqlx::query("SELECT t.id,t.title,t.created_at,t.updated_at,u.login AS author,COUNT(p.id) AS posts FROM forum_topics t JOIN users u ON u.id=t.author_id LEFT JOIN forum_posts p ON p.topic_id=t.id GROUP BY t.id,u.login ORDER BY t.updated_at DESC").fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"title":r.get::<String,_>("title"),"author":r.get::<String,_>("author"),"posts":r.get::<i64,_>("posts"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(topics))
}
async fn create_forum_topic(
    s: web::Data<AppState>,
    u: CurrentUser,
    input: web::Json<ForumTopic>,
) -> Result<HttpResponse, ApiError> {
    let title = input.title.trim();
    let content = input.content.trim();
    if title.is_empty() || title.len() > 120 || content.is_empty() {
        return Err(ApiError::BadRequest("invalid forum topic".into()));
    }
    let mut tx = s.pool.begin().await?;
    let id: i64 =
        sqlx::query_scalar("INSERT INTO forum_topics(title,author_id) VALUES($1,$2) RETURNING id")
            .bind(title)
            .bind(u.0)
            .fetch_one(&mut *tx)
            .await?;
    sqlx::query("INSERT INTO forum_posts(topic_id,author_id,content) VALUES($1,$2,$3)")
        .bind(id)
        .bind(u.0)
        .bind(content)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"id":id})))
}
async fn forum_topic(
    s: web::Data<AppState>,
    _: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let topic=sqlx::query("SELECT t.id,t.title,u.login AS author,t.created_at FROM forum_topics t JOIN users u ON u.id=t.author_id WHERE t.id=$1").bind(*id).fetch_optional(&s.pool).await?.ok_or(ApiError::NotFound)?;
    let posts=sqlx::query("SELECT p.id,p.content,p.created_at,u.login AS author FROM forum_posts p JOIN users u ON u.id=p.author_id WHERE p.topic_id=$1 ORDER BY p.created_at").bind(*id).fetch_all(&s.pool).await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"content":r.get::<String,_>("content"),"author":r.get::<String,_>("author"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(serde_json::json!({"id":topic.get::<i64,_>("id"),"title":topic.get::<String,_>("title"),"author":topic.get::<String,_>("author"),"created_at":topic.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"posts":posts})))
}
async fn create_forum_post(
    s: web::Data<AppState>,
    u: CurrentUser,
    id: web::Path<i64>,
    input: web::Json<ForumPost>,
) -> Result<HttpResponse, ApiError> {
    let content = input.content.trim();
    if content.is_empty() {
        return Err(ApiError::BadRequest("empty forum post".into()));
    }
    let mut tx = s.pool.begin().await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM forum_topics WHERE id=$1)")
        .bind(*id)
        .fetch_one(&mut *tx)
        .await?;
    if !exists {
        return Err(ApiError::NotFound);
    }
    let post: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts(topic_id,author_id,content) VALUES($1,$2,$3) RETURNING id",
    )
    .bind(*id)
    .bind(u.0)
    .bind(content)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("UPDATE forum_topics SET updated_at=now() WHERE id=$1")
        .bind(*id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"id":post})))
}
fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 120
        && slug
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'_' | b'-'))
}
async fn page_data(
    _: web::Data<AppState>,
    u: CurrentUser,
    page: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let known = [
        "software",
        "internet",
        "processes",
        "missions",
        "hardware",
        "clan",
        "list",
        "finances",
        "university",
        "ranking",
        "log",
        "fame",
        "doom",
        "bitcoin",
        "certs",
        "mail",
        "profile",
        "settings",
        "research",
        "riddle",
        "war",
        "webserver",
        "stats",
    ];
    if !known.contains(&page.as_str()) {
        return Err(ApiError::NotFound);
    }
    Ok(HttpResponse::Ok()
        .json(serde_json::json!({"page":page.into_inner(),"user_id":u.0,"status":"available"})))
}
