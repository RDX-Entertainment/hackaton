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
    let v=rows(&s,u.0,"SELECT pid,victim_id,action,info,info_str,time_ideal,time_worked,is_paused FROM processes WHERE creator_id=$1 ORDER BY time_start DESC").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("pid"),"victim_id":r.get::<i64,_>("victim_id"),"action":r.get::<i16,_>("action"),"info":r.get::<String,_>("info"),"details":r.get::<String,_>("info_str"),"ideal_seconds":r.get::<i32,_>("time_ideal"),"worked_seconds":r.get::<i32,_>("time_worked"),"paused":r.get::<bool,_>("is_paused")})).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(v))
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
    let v=rows(&s,u.0,"SELECT id,type,status,hirer,victim,info,info2,prize,level FROM missions WHERE user_id=$1 ORDER BY date_generated DESC").await?.into_iter().map(|r|serde_json::json!({"id":r.get::<i64,_>("id"),"type":r.get::<i16,_>("type"),"status":r.get::<i16,_>("status"),"hirer":r.get::<i64,_>("hirer"),"victim":r.get::<i64,_>("victim"),"info":r.get::<i64,_>("info"),"info2":r.get::<i64,_>("info2"),"prize":r.get::<i64,_>("prize"),"level":r.get::<i16,_>("level")})).collect::<Vec<_>>();
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
