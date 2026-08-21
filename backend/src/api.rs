use crate::{
    auth::{CurrentUser, issue_token},
    error::ApiError,
    legacy::PROCESS_ACTIONS,
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
    actions: Vec<ProcessAction>,
}
#[derive(Serialize)]
struct ProcessAction {
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
fn actions() -> Vec<ProcessAction> {
    PROCESS_ACTIONS
        .iter()
        .map(|(name, id)| ProcessAction { name, id: *id })
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
            .route("/software", web::get().to(software))
            .route("/hardware", web::get().to(hardware))
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
async fn rows(s: &AppState, u: i64, sql: &str) -> Result<Vec<PgRow>, ApiError> {
    Ok(sqlx::query(sql).bind(u).fetch_all(&s.pool).await?)
}
async fn software(s: web::Data<AppState>, u: CurrentUser) -> Result<HttpResponse, ApiError> {
    let v=rows(&s,u.0,"SELECT id,soft_name,soft_version,soft_size,soft_ram,soft_type,soft_hidden,is_folder FROM software WHERE user_id=$1 AND NOT is_npc ORDER BY soft_name").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"name":r.get::<String,_>("soft_name"),"version":r.get::<i32,_>("soft_version"),"size":r.get::<i32,_>("soft_size"),"ram":r.get::<i32,_>("soft_ram"),"type":r.get::<i16,_>("soft_type"),"hidden":r.get::<bool,_>("soft_hidden"),"folder":r.get::<bool,_>("is_folder")})).collect::<Vec<_>>();
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
    let mut tx = s.pool.begin().await?;
    let software=sqlx::query("SELECT id,soft_name,soft_version,soft_type FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc FOR UPDATE").bind(i.software_id).bind(u.0).fetch_optional(&mut *tx).await?.ok_or(ApiError::NotFound)?;
    let version = software.get::<i32, _>("soft_version") + 1;
    let id:i64=sqlx::query_scalar("INSERT INTO research_history(user_id,software_id,software_type,new_version,software_name) VALUES($1,$2,$3,$4,$5) RETURNING id").bind(u.0).bind(i.software_id).bind(software.get::<i16,_>("soft_type")).bind(version).bind(software.get::<String,_>("soft_name")).fetch_one(&mut *tx).await?;
    sqlx::query("UPDATE software SET soft_version=$1 WHERE id=$2")
        .bind(version)
        .bind(i.software_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE users_stats SET money_research=money_research+1 WHERE uid=$1")
        .bind(u.0)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Created()
        .json(serde_json::json!({"id":id,"software_id":i.software_id,"version":version})))
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
