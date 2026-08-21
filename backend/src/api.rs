use actix_web::{HttpResponse, web};
use bcrypt::{hash, verify};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sqlx::{Row, mysql::MySqlRow};

use crate::{
    auth::{CurrentUser, issue_token},
    error::ApiError,
    legacy::PROCESS_ACTIONS,
    state::AppState,
};

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

fn user_from(row: &MySqlRow) -> User {
    User {
        id: row.get("id"),
        login: row.get("login"),
        email: row.get("email"),
        game_ip: row.get("gameIP"),
        premium: row.get::<i8, _>("premium") != 0,
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
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
            .route("/mails", web::get().to(mails))
            .route("/mails", web::post().to(compose_mail))
            .route("/mails/{id}", web::get().to(mail))
            .route("/mails/{id}", web::delete().to(delete_mail))
            .route("/missions", web::get().to(missions))
            .route("/missions/{id}", web::post().to(mission_command))
            .route("/finances/accounts", web::get().to(bank_accounts))
            .route("/finances/transfers", web::post().to(bank_transfer))
            .route("/ranking/users", web::get().to(user_ranking))
            .route("/friends", web::get().to(friends))
            .route("/friends/requests", web::post().to(request_friend))
            .route(
                "/friends/requests/{id}/accept",
                web::post().to(accept_friend),
            )
            .route("/pages/{page}", web::get().to(page_data)),
    );
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok"}))
}

async fn login(
    state: web::Data<AppState>,
    input: web::Json<Login>,
) -> Result<HttpResponse, ApiError> {
    let row = sqlx::query("SELECT id, login, email, gameIP, premium, password FROM users WHERE BINARY login = ? LIMIT 1")
        .bind(&input.username).fetch_optional(&state.pool).await?.ok_or(ApiError::Unauthorized)?;
    let hash: String = row.get("password");
    verify(&input.password, &hash)
        .map_err(|_| ApiError::Unauthorized)?
        .then_some(())
        .ok_or(ApiError::Unauthorized)?;
    sqlx::query("UPDATE users SET lastLogin = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(row.get::<i64, _>("id"))
        .execute(&state.pool)
        .await?;
    let token = issue_token(row.get("id"), &state.jwt_secret)?;
    Ok(HttpResponse::Ok().json(Session {
        token,
        user: user_from(&row),
    }))
}

async fn register(
    state: web::Data<AppState>,
    input: web::Json<Register>,
) -> Result<HttpResponse, ApiError> {
    let username = input.username.trim();
    if !(1..=15).contains(&username.len())
        || !username
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    {
        return Err(ApiError::BadRequest(
            "invalid username; allowed characters are azAZ09._- and 15 characters maximum".into(),
        ));
    }
    if input.password.is_empty() || !input.email.contains('@') || input.email.len() > 50 {
        return Err(ApiError::BadRequest("invalid registration data".into()));
    }
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE login = ? OR email = ?")
        .bind(username)
        .bind(&input.email)
        .fetch_one(&state.pool)
        .await?;
    if exists != 0 {
        return Err(ApiError::BadRequest(
            "username or email already exists".into(),
        ));
    }
    let game_ip = rand::rng().random_range(1_i64..=u32::MAX as i64);
    let game_pass: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    // The legacy BCrypt implementation used cost 13 and stores 60-character $2a$ hashes.
    let hash = hash(&input.password, 13)
        .map_err(|_| ApiError::BadRequest("cannot hash password".into()))?;
    let mut tx = state.pool.begin().await?;
    let result = sqlx::query("INSERT INTO users (login,password,email,gamePass,gameIP,realIP,homeIP,learning,premium) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0)")
        .bind(username).bind(hash).bind(&input.email).bind(game_pass).bind(game_ip).bind(game_ip).bind(game_ip).execute(&mut *tx).await?;
    let user_id = result.last_insert_id() as i64;
    sqlx::query("INSERT INTO users_stats (uid,dateJoined,exp,certifications,awards,timePlaying,missionCount,hackCount,ddosCount,warezSent,spamSent,bitcoinSent,ipResets,pwdResets,moneyEarned,moneyTransfered,moneyHardware,moneyResearch,profileViews) VALUES (?,NOW(),0,'','',0,0,0,0,0,0,0,0,0,0,0,0,0,0)")
        .bind(user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO hardware (userID, name, cpu, hdd, ram, net, isNPC) VALUES (?, 'Server #1', 500, 100, 256, 1, 0)").bind(user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO `log` (userID, text, isNPC) VALUES (?, CONCAT(SUBSTRING(NOW(), 1, 16), ' - localhost installed current operating system'), 0)").bind(user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO cache (userID, reputation) VALUES (?, 0)")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO cache_profile (userID, expireDate) VALUES (?, NOW())")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO ranking_user (userID, rank) VALUES (?, -1)")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO certifications (userID, certLevel) VALUES (?, 0)")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO users_puzzle (userID, puzzleID) VALUES (?, 0)")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO users_learning (userID, learning) VALUES (?, 1)")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO users_language (userID, lang) VALUES (?, 'en')")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    let row = sqlx::query("SELECT id, login, email, gameIP, premium FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
    let token = issue_token(user_id, &state.jwt_secret)?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(Session {
        token,
        user: user_from(&row),
    }))
}

async fn me(state: web::Data<AppState>, user: CurrentUser) -> Result<HttpResponse, ApiError> {
    let row = sqlx::query("SELECT id, login, email, gameIP, premium FROM users WHERE id = ?")
        .bind(user.0)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(HttpResponse::Ok().json(user_from(&row)))
}

async fn dashboard(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let row = sqlx::query("SELECT id, login, email, gameIP, premium FROM users WHERE id = ?")
        .bind(user.0)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::NotFound)?;
    let money: Option<f64> =
        sqlx::query_scalar("SELECT SUM(cash) FROM bankAccounts WHERE bankUser = ?")
            .bind(user.0)
            .fetch_one(&state.pool)
            .await?;
    let processes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM processes WHERE pCreatorID = ?")
        .bind(user.0)
        .fetch_one(&state.pool)
        .await?;
    let unread_mail: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM mails WHERE `to` = ? AND isRead = 0")
            .bind(user.0)
            .fetch_one(&state.pool)
            .await?;
    Ok(HttpResponse::Ok().json(Dashboard {
        user: user_from(&row),
        money: money.unwrap_or(0.0),
        processes,
        unread_mail,
        actions: PROCESS_ACTIONS
            .iter()
            .map(|(name, id)| ProcessAction { name, id: *id })
            .collect(),
    }))
}

async fn process_actions(_: CurrentUser) -> HttpResponse {
    HttpResponse::Ok().json(
        PROCESS_ACTIONS
            .iter()
            .map(|(name, id)| ProcessAction { name, id: *id })
            .collect::<Vec<_>>(),
    )
}

async fn software(state: web::Data<AppState>, user: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT id, softName, softVersion, softSize, softRam, softType, softHidden, isFolder FROM software WHERE userID = ? AND isNPC = 0 ORDER BY softName")
        .bind(user.0).fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r| serde_json::json!({"id":r.get::<i64,_>("id"),"name":r.get::<String,_>("softName"),"version":r.get::<i64,_>("softVersion"),"size":r.get::<i64,_>("softSize"),"ram":r.get::<i64,_>("softRam"),"type":r.get::<i64,_>("softType"),"hidden":r.get::<i8,_>("softHidden") != 0,"folder":r.get::<i8,_>("isFolder") != 0})).collect::<Vec<_>>()))
}

async fn hardware(state: web::Data<AppState>, user: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT serverID, name, cpu, hdd, ram, net FROM hardware WHERE userID = ? AND isNPC = 0 ORDER BY serverID")
        .bind(user.0).fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r| serde_json::json!({"id":r.get::<i64,_>("serverID"),"name":r.get::<String,_>("name"),"cpu":r.get::<f32,_>("cpu"),"hdd":r.get::<f32,_>("hdd"),"ram":r.get::<f32,_>("ram"),"net":r.get::<f32,_>("net")})).collect::<Vec<_>>()))
}

async fn processes(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT pid, pVictimID, pAction, pInfo, pInfoStr, pTimeStart, pTimeEnd, pTimeIdeal, pTimeWorked, isPaused FROM processes WHERE pCreatorID = ? ORDER BY pTimeStart DESC")
        .bind(user.0).fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r| serde_json::json!({"id":r.get::<i64,_>("pid"),"victim_id":r.get::<i64,_>("pVictimID"),"action":r.get::<i64,_>("pAction"),"info":r.get::<String,_>("pInfo"),"details":r.get::<String,_>("pInfoStr"),"ideal_seconds":r.get::<i64,_>("pTimeIdeal"),"worked_seconds":r.get::<i64,_>("pTimeWorked"),"paused":r.get::<i8,_>("isPaused") != 0})).collect::<Vec<_>>()))
}

async fn delete_process(
    state: web::Data<AppState>,
    user: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    let row =
        sqlx::query("SELECT pCreatorID, pAction, pInfo FROM processes WHERE pid = ? FOR UPDATE")
            .bind(*id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(ApiError::NotFound)?;
    if row.get::<i64, _>("pCreatorID") != user.0 {
        return Err(ApiError::NotFound);
    }
    if row.get::<i64, _>("pAction") == 8 {
        sqlx::query("DELETE FROM log_edit WHERE id = CAST(? AS UNSIGNED)")
            .bind(row.get::<String, _>("pInfo"))
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM processes_paused WHERE pid = ?")
        .bind(*id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM processes WHERE pid = ?")
        .bind(*id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn mails(state: web::Data<AppState>, user: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT id, `from`, `to`, type, subject, text, dateSent, isRead, isDeleted FROM mails WHERE (`from` = ? OR `to` = ?) AND isDeleted = 0 ORDER BY dateSent DESC")
        .bind(user.0).bind(user.0).fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r| serde_json::json!({"id":r.get::<i64,_>("id"),"from":r.get::<i64,_>("from"),"to":r.get::<i64,_>("to"),"type":r.get::<i64,_>("type"),"subject":r.get::<String,_>("subject"),"text":r.get::<String,_>("text"),"read":r.get::<i8,_>("isRead") != 0})).collect::<Vec<_>>()))
}

async fn mail(
    state: web::Data<AppState>,
    user: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let row = sqlx::query("SELECT id, `from`, `to`, type, subject, text, dateSent, isRead, isDeleted FROM mails WHERE id = ? AND (`from` = ? OR `to` = ?) LIMIT 1")
        .bind(*id).bind(user.0).bind(user.0).fetch_optional(&state.pool).await?.ok_or(ApiError::NotFound)?;
    if row.get::<i8, _>("isDeleted") != 0 && row.get::<i64, _>("from") != user.0 {
        return Err(ApiError::NotFound);
    }
    if row.get::<i64, _>("to") == user.0 {
        sqlx::query("UPDATE mails SET isRead = 1 WHERE id = ?")
            .bind(*id)
            .execute(&state.pool)
            .await?;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"id":row.get::<i64,_>("id"),"from":row.get::<i64,_>("from"),"to":row.get::<i64,_>("to"),"type":row.get::<i64,_>("type"),"subject":row.get::<String,_>("subject"),"text":row.get::<String,_>("text"),"read":true})))
}

async fn compose_mail(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<ComposeMail>,
) -> Result<HttpResponse, ApiError> {
    let subject = input.subject.trim();
    let text = input.text.trim();
    if subject.is_empty()
        || subject.len() > 40
        || !subject
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".,!? _-$()'\"".contains(&c))
    {
        return Err(ApiError::BadRequest("invalid mail subject".into()));
    }
    if text.is_empty() {
        return Err(ApiError::BadRequest("message can not be empty".into()));
    }
    let recipient = if let Ok(id) = input.to.trim().parse::<i64>() {
        sqlx::query("SELECT id FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?
            .map(|r| r.get::<i64, _>("id"))
            .ok_or(ApiError::NotFound)?
    } else {
        let login = input.to.trim();
        sqlx::query("SELECT id FROM users WHERE BINARY login = ?")
            .bind(login)
            .fetch_optional(&state.pool)
            .await?
            .map(|r| r.get::<i64, _>("id"))
            .ok_or(ApiError::NotFound)?
    };
    if recipient == user.0 {
        return Err(ApiError::BadRequest(
            "you can not send email to yourself".into(),
        ));
    }
    let result = sqlx::query("INSERT INTO mails (`from`, `to`, type, subject, text, dateSent, isRead, isDeleted) VALUES (?, ?, 0, ?, ?, NOW(), 0, 0)")
        .bind(user.0).bind(recipient).bind(subject).bind(text).execute(&state.pool).await?;
    Ok(HttpResponse::Created()
        .json(serde_json::json!({"id":result.last_insert_id(),"to":recipient})))
}

async fn delete_mail(
    state: web::Data<AppState>,
    user: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let result =
        sqlx::query("UPDATE mails SET isDeleted = 1 WHERE id = ? AND `to` = ? AND isDeleted = 0")
            .bind(*id)
            .bind(user.0)
            .execute(&state.pool)
            .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(HttpResponse::NoContent().finish())
}

async fn missions(state: web::Data<AppState>, user: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT id, type, status, hirer, victim, info, info2, missionStart, missionEnd, prize, level FROM missions WHERE userID = ? ORDER BY dateGenerated DESC")
        .bind(user.0).fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r| serde_json::json!({"id":r.get::<i64,_>("id"),"type":r.get::<i64,_>("type"),"status":r.get::<i64,_>("status"),"hirer":r.get::<i64,_>("hirer"),"victim":r.get::<i64,_>("victim"),"info":r.get::<i64,_>("info"),"info2":r.get::<i64,_>("info2"),"prize":r.get::<i64,_>("prize"),"level":r.get::<i64,_>("level")})).collect::<Vec<_>>()))
}

async fn mission_command(
    state: web::Data<AppState>,
    user: CurrentUser,
    id: web::Path<i64>,
    input: web::Json<MissionCommand>,
) -> Result<HttpResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    let mission =
        sqlx::query("SELECT id, type, status, userID, level FROM missions WHERE id = ? FOR UPDATE")
            .bind(*id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(ApiError::NotFound)?;
    match input.action.as_str() {
        "accept" => {
            if mission.get::<i64, _>("status") != 1 {
                return Err(ApiError::BadRequest(
                    "mission has already been taken".into(),
                ));
            }
            let active: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM missions WHERE userID = ? AND status = 2")
                    .bind(user.0)
                    .fetch_one(&mut *tx)
                    .await?;
            if active != 0 {
                return Err(ApiError::BadRequest("you already are in a mission".into()));
            }
            let version: Option<i64> = sqlx::query_scalar("SELECT MAX(softVersion) FROM software WHERE userID = ? AND isNPC = 0 AND softType = 1")
                .bind(user.0).fetch_one(&mut *tx).await?;
            let level = match version.unwrap_or(0) {
                60.. => 3,
                30.. => 2,
                _ => 1,
            };
            if mission.get::<i64, _>("level") != level {
                return Err(ApiError::BadRequest(
                    "mission level does not match your best cracker".into(),
                ));
            }
            sqlx::query("UPDATE missions SET status = 2, userID = ? WHERE id = ?")
                .bind(user.0)
                .bind(*id)
                .execute(&mut *tx)
                .await?;
        }
        "abort" => {
            if mission.get::<i64, _>("userID") != user.0 || mission.get::<i64, _>("status") != 2 {
                return Err(ApiError::BadRequest("you are not in this mission".into()));
            }
            if mission.get::<i64, _>("type") > 49 {
                return Err(ApiError::BadRequest(
                    "you are not allowed to abort this kind of mission".into(),
                ));
            }
            sqlx::query("UPDATE missions SET status = 4 WHERE id = ?")
                .bind(*id)
                .execute(&mut *tx)
                .await?;
        }
        _ => return Err(ApiError::BadRequest("unsupported mission action".into())),
    }
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"id":*id,"action":input.action})))
}

async fn bank_accounts(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT id, bankAcc, bankID, cash, dateCreated FROM bankAccounts WHERE bankUser = ? ORDER BY dateCreated DESC")
        .bind(user.0).fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r| serde_json::json!({"id":r.get::<i64,_>("id"),"account":r.get::<i64,_>("bankAcc"),"bank_id":r.get::<i64,_>("bankID"),"cash":r.get::<i64,_>("cash")})).collect::<Vec<_>>()))
}

async fn bank_transfer(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<BankTransfer>,
) -> Result<HttpResponse, ApiError> {
    if input.amount <= 0 || input.from_account == input.to_account {
        return Err(ApiError::BadRequest("invalid transfer".into()));
    }
    let mut tx = state.pool.begin().await?;
    let source = sqlx::query("SELECT id, bankAcc, bankPass, bankID, bankUser, cash FROM bankAccounts WHERE bankAcc = ? FOR UPDATE")
        .bind(input.from_account)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError::NotFound)?;
    if source.get::<i64, _>("bankUser") != user.0
        || source.get::<String, _>("bankPass") != input.password
    {
        return Err(ApiError::Unauthorized);
    }
    let target = sqlx::query(
        "SELECT id, bankAcc, bankID, bankUser FROM bankAccounts WHERE bankAcc = ? FOR UPDATE",
    )
    .bind(input.to_account)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;
    if source.get::<i64, _>("cash") < input.amount {
        return Err(ApiError::BadRequest("insufficient funds".into()));
    }
    sqlx::query("UPDATE bankAccounts SET cash = cash - ? WHERE id = ?")
        .bind(input.amount)
        .bind(source.get::<i64, _>("id"))
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE bankAccounts SET cash = cash + ? WHERE id = ?")
        .bind(input.amount)
        .bind(target.get::<i64, _>("id"))
        .execute(&mut *tx)
        .await?;
    // This matches Ranking::updateMoneyStats(3, amount, userTo) in the legacy transfer flow.
    sqlx::query("UPDATE users_stats SET moneyTransfered = moneyTransfered + ? WHERE uid = ?")
        .bind(input.amount)
        .bind(target.get::<i64, _>("bankUser"))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(
        serde_json::json!({"from":input.from_account,"to":input.to_account,"amount":input.amount}),
    ))
}

async fn user_ranking(
    state: web::Data<AppState>,
    _: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT r.rank, u.id, u.login FROM ranking_user r JOIN users u ON u.id = r.userID ORDER BY r.rank ASC LIMIT 100")
        .fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r| serde_json::json!({"rank":r.get::<i64,_>("rank"),"user_id":r.get::<i64,_>("id"),"login":r.get::<String,_>("login")})).collect::<Vec<_>>()))
}

async fn friends(state: web::Data<AppState>, user: CurrentUser) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query("SELECT u.id, u.login, f.dateAdd FROM users_friends f JOIN users u ON u.id = CASE WHEN f.userID = ? THEN f.friendID ELSE f.userID END WHERE f.userID = ? OR f.friendID = ? ORDER BY f.dateAdd DESC")
        .bind(user.0).bind(user.0).bind(user.0).fetch_all(&state.pool).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|r| serde_json::json!({"id":r.get::<i64,_>("id"),"login":r.get::<String,_>("login")})).collect::<Vec<_>>()))
}

async fn request_friend(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<FriendRequest>,
) -> Result<HttpResponse, ApiError> {
    if input.user_id == user.0 {
        return Err(ApiError::BadRequest("you can not add yourself".into()));
    }
    let mut tx = state.pool.begin().await?;
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(input.user_id)
        .fetch_one(&mut *tx)
        .await?;
    if exists == 0 {
        return Err(ApiError::NotFound);
    }
    let related: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users_friends WHERE (userID = ? AND friendID = ?) OR (userID = ? AND friendID = ?)").bind(user.0).bind(input.user_id).bind(input.user_id).bind(user.0).fetch_one(&mut *tx).await?;
    if related != 0 {
        return Err(ApiError::BadRequest("already friends".into()));
    }
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friend_requests WHERE userID = ? AND requestedBy = ?",
    )
    .bind(input.user_id)
    .bind(user.0)
    .fetch_one(&mut *tx)
    .await?;
    if pending != 0 {
        return Err(ApiError::BadRequest("request already sent".into()));
    }
    let result = sqlx::query("INSERT INTO friend_requests (userID, requestedBy) VALUES (?, ?)")
        .bind(input.user_id)
        .bind(user.0)
        .execute(&mut *tx)
        .await?;
    let sender: String = sqlx::query_scalar("SELECT login FROM users WHERE id = ?")
        .bind(user.0)
        .fetch_one(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO mails (`from`, `to`, type, subject, text, dateSent, isRead, isDeleted) VALUES (-6, ?, 1, ?, ?, NOW(), 0, 0)")
        .bind(input.user_id).bind(format!("New friend request from {sender}")).bind(format!("{sender} added you as a friend. Request #{}.", result.last_insert_id())).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"id":result.last_insert_id()})))
}

async fn accept_friend(
    state: web::Data<AppState>,
    user: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    let row = sqlx::query(
        "SELECT requestedBy FROM friend_requests WHERE id = ? AND userID = ? FOR UPDATE",
    )
    .bind(*id)
    .bind(user.0)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;
    let friend_id: i64 = row.get("requestedBy");
    sqlx::query("DELETE FROM friend_requests WHERE id = ?")
        .bind(*id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO users_friends (userID, friendID, dateAdd) VALUES (?, ?, NOW())")
        .bind(user.0)
        .bind(friend_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"friend_id":friend_id})))
}

async fn page_data(
    _state: web::Data<AppState>,
    user: CurrentUser,
    page: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    // Page data starts with an authenticated identity. Domain-specific responses are added per legacy page.
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
        .json(serde_json::json!({"page":page.into_inner(),"user_id":user.0,"status":"available"})))
}
