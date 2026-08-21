use dotenvy::dotenv;
use sqlx::{Row, postgres::PgPoolOptions};
use std::{env, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let once = env::args().any(|arg| arg == "--once");
    loop {
        tick(&pool).await?;
        if once {
            break;
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
    Ok(())
}

async fn tick(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    resolve_completed(pool).await?;
    // Port of cron/generateMissions.php: replenish the public board from NPCs.
    for (level, desired) in [(1_i16, 50_i64), (2, 30), (3, 25)] {
        sqlx::query("WITH available AS (SELECT COUNT(*) AS total FROM missions WHERE status=1 AND level=$1), candidates AS (SELECT h.npc_ip AS hirer,v.npc_ip AS victim FROM npc h JOIN npc v ON h.id<>v.id WHERE h.npc_type=70+$1 AND v.npc_type=70+$1 AND h.down_until<=now() AND v.down_until<=now() LIMIT (SELECT GREATEST(0,$2-total) FROM available)) INSERT INTO missions(type,status,hirer,victim,prize,level) SELECT 1,1,hirer,victim,(150*$1)::bigint,$1 FROM candidates")
            .bind(level).bind(desired).execute(pool).await?;
    }
    // Port of cron/updatePremium.php.
    sqlx::query("UPDATE users SET premium=false WHERE id IN (SELECT user_id FROM users_premium WHERE premium_until < now())")
        .execute(pool).await?;
    sqlx::query("DELETE FROM users_premium WHERE premium_until < now()")
        .execute(pool)
        .await?;
    // Port of cron2/updateRanking.py, ranking users by their accumulated experience.
    sqlx::query("INSERT INTO ranking_user(user_id,rank) SELECT uid,ROW_NUMBER() OVER(ORDER BY exp DESC,uid)::integer FROM users_stats ON CONFLICT(user_id) DO UPDATE SET rank=EXCLUDED.rank")
        .execute(pool).await?;
    // Port of cron/updateServerStats.php; JSON keeps this snapshot extensible.
    sqlx::query("INSERT INTO system_stats(key,payload,updated_at) VALUES('server',jsonb_build_object('users',(SELECT count(*) FROM users),'active_users',(SELECT count(*) FROM users WHERE last_login >= now()-interval '14 days'),'clans',(SELECT count(*) FROM clans),'clan_members',(SELECT count(*) FROM clan_members),'mails',(SELECT count(*) FROM mails WHERE sender_id>0),'missions',(SELECT count(*) FROM missions),'research',(SELECT count(*) FROM research_history),'processes',(SELECT count(*) FROM processes),'software',(SELECT count(*) FROM software),'servers',(SELECT count(*) FROM hardware),'money',COALESCE((SELECT sum(cash) FROM bank_accounts),0)),now()) ON CONFLICT(key) DO UPDATE SET payload=EXCLUDED.payload,updated_at=EXCLUDED.updated_at")
        .execute(pool).await?;
    Ok(())
}

/// Port of Process::completeProcess, which the legacy source itself calls the most
/// important function of the game: the one that applies a finished process's side effects.
///
/// Until now this was a bare completion marker, so a finished HACK expired without handing
/// the player anything. Only the two actions this port can currently start are resolved;
/// every other action still just gets marked done, exactly as before.
async fn resolve_completed(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let due = sqlx::query("SELECT pid,creator_id,victim_id,action,info FROM processes WHERE completed_at IS NULL AND NOT is_paused AND time_end<=now()")
        .fetch_all(pool).await?;
    for p in due {
        let pid: i64 = p.get("pid");
        let user: i64 = p.get("creator_id");
        let victim: i64 = p.get("victim_id");
        let action: i16 = p.get("action");
        // Internet::hack writes "npc" or "player" into info when it starts the process,
        // which is what the legacy carried in its isNPC flag.
        let is_npc = p.get::<String, _>("info") == "npc";
        match action {
            8 => edit_log_effect(pool, pid, victim, is_npc).await?,
            11 => hack_effect(pool, user, victim, is_npc).await?,
            15 => scan_effect(pool, user, victim, is_npc).await?,
            _ => {}
        }
        sqlx::query("UPDATE processes SET time_worked=time_ideal,completed_at=now() WHERE pid=$1")
            .bind(pid)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Resolves the IP of a finished process's victim, plus the password a crack would reveal.
/// Returns None when the victim disappeared while the process was running.
async fn victim_ip(
    pool: &sqlx::PgPool,
    victim: i64,
    is_npc: bool,
) -> Result<Option<(i64, String)>, sqlx::Error> {
    let sql = if is_npc {
        "SELECT npc_ip AS ip,npc_pass AS pass FROM npc WHERE id=$1"
    } else {
        "SELECT game_ip AS ip,game_pass AS pass FROM users WHERE id=$1"
    };
    Ok(sqlx::query(sql)
        .bind(victim)
        .fetch_optional(pool)
        .await?
        .map(|r| (r.get("ip"), r.get("pass"))))
}

/// Case 11 of completeProcess. A successful brute force lands the victim's credentials in
/// the player's compromised list, opens the machine and pays experience.
async fn hack_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    is_npc: bool,
) -> Result<(), sqlx::Error> {
    let Some((ip, pass)) = victim_ip(pool, victim, is_npc).await? else {
        return Ok(());
    };
    // addToList($_SESSION['id'], $ip, 'root', $pass, '0', '0'). The legacy refused a repeat
    // with ALREADY_LISTED; compromised_lists has no unique key, so rather than pile up
    // duplicates a repeat hack refreshes the row already there.
    let known: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM compromised_lists WHERE user_id=$1 AND ip=$2)",
    )
    .bind(user)
    .bind(ip)
    .fetch_one(pool)
    .await?;
    if known {
        sqlx::query("UPDATE compromised_lists SET password=$3,hacked_at=now() WHERE user_id=$1 AND ip=$2")
            .bind(user).bind(ip).bind(&pass).execute(pool).await?;
    } else {
        sqlx::query(
            "INSERT INTO compromised_lists(user_id,ip,login,password) VALUES($1,$2,'root',$3)",
        )
        .bind(user)
        .bind(ip)
        .bind(&pass)
        .execute(pool)
        .await?;
    }
    // The crack is also what opens the machine, so the player need not log in again.
    sqlx::query("INSERT INTO internet_access(user_id,ip,via_exploit) VALUES($1,$2,false) ON CONFLICT(user_id,ip) DO UPDATE SET granted_at=now()")
        .bind(user).bind(ip).execute(pool).await?;
    // exp_add('HACK', hasher version + victim cracker version), as in the legacy.
    let exp:i64=sqlx::query_scalar("SELECT COALESCE(sum(soft_version),0)::bigint FROM software WHERE user_id=$1 AND soft_type IN (1,2) AND is_npc=$2")
        .bind(victim).bind(is_npc).fetch_one(pool).await?;
    sqlx::query("UPDATE users_stats SET exp=exp+$2,hack_count=hack_count+1 WHERE uid=$1")
        .bind(user)
        .bind(exp)
        .execute(pool)
        .await?;
    // Every action in completeProcess wrote two log lines, one on each side, through
    // LogVPC::logText('LOGIN', ...). The victim sees who came in, the attacker sees where
    // they went, and the pair is what makes log editing worth doing at all.
    if let Some(attacker_ip) = sqlx::query_scalar::<_, i64>("SELECT game_ip FROM users WHERE id=$1").bind(user).fetch_optional(pool).await? {
        add_log(pool, victim, is_npc, &format!("[{}] logged in as root", dotted(attacker_ip))).await?;
        add_log(pool, user, false, &format!("localhost logged in to [{}] as root", dotted(ip))).await?;
    }
    Ok(())
}

/// Port of LogVPC::addLog. The legacy stamped every line with the minute it happened and
/// prepended it, so a machine keeps one log row with the newest entry on top.
async fn add_log(
    pool: &sqlx::PgPool,
    owner: i64,
    is_npc: bool,
    text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE log SET text=to_char(now(),'YYYY-MM-DD HH24:MI')||' - '||$3||E'
'||text WHERE user_id=$1 AND is_npc=$2")
        .bind(owner).bind(is_npc).bind(text).execute(pool).await?;
    Ok(())
}

/// long2ip. Game IPs travel as the 32-bit integers the legacy stored, but a log line is
/// read by a person.
fn dotted(ip: i64) -> String {
    format!(
        "{}.{}.{}.{}",
        (ip >> 24) & 255,
        (ip >> 16) & 255,
        (ip >> 8) & 255,
        ip & 255
    )
}

/// Case 8 of completeProcess. The replacement text waited in log_edit while the process
/// ran; finishing it is what actually overwrites the victim's log.
async fn edit_log_effect(
    pool: &sqlx::PgPool,
    pid: i64,
    victim: i64,
    is_npc: bool,
) -> Result<(), sqlx::Error> {
    let Some(text) = sqlx::query_scalar::<_, String>("SELECT log_text FROM log_edit WHERE process_id=$1 ORDER BY id DESC LIMIT 1").bind(pid).fetch_optional(pool).await? else {
        return Ok(());
    };
    sqlx::query("UPDATE log SET text=$3 WHERE user_id=$1 AND is_npc=$2")
        .bind(victim)
        .bind(is_npc)
        .bind(text)
        .execute(pool)
        .await?;
    Ok(())
}

/// Case 15 of completeProcess. The legacy only set $_SESSION['PORT_SCAN'] so the next
/// render could list the open ports; persisted, the scan outlives the request.
async fn scan_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    is_npc: bool,
) -> Result<(), sqlx::Error> {
    let Some((ip, _)) = victim_ip(pool, victim, is_npc).await? else {
        return Ok(());
    };
    sqlx::query("INSERT INTO internet_scan(user_id,ip) VALUES($1,$2) ON CONFLICT(user_id,ip) DO UPDATE SET scanned_at=now()")
        .bind(user).bind(ip).execute(pool).await?;
    Ok(())
}
