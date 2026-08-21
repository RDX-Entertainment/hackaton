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
    // Port of cron/restoreSoftware.php: put back any original software an NPC has lost.
    // Without this a machine stripped by a DELETE stays stripped for good, and the world
    // wears down to nothing worth breaking into.
    sqlx::query("INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,is_npc) SELECT o.npc_id,o.soft_name,o.soft_version,o.soft_size,o.soft_ram,o.soft_type,TRUE FROM software_original o WHERE NOT EXISTS(SELECT 1 FROM software s WHERE s.user_id=o.npc_id AND s.is_npc AND s.soft_name=o.soft_name AND s.soft_version=o.soft_version)")
        .execute(pool).await?;
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
    let due = sqlx::query("SELECT pid,creator_id,victim_id,action,soft_id,info FROM processes WHERE completed_at IS NULL AND NOT is_paused AND time_end<=now()")
        .fetch_all(pool).await?;
    for p in due {
        let pid: i64 = p.get("pid");
        let user: i64 = p.get("creator_id");
        let victim: i64 = p.get("victim_id");
        let action: i16 = p.get("action");
        let soft: i64 = p.get("soft_id");
        // Internet::hack writes "npc" or "player" into info when it starts the process,
        // which is what the legacy carried in its isNPC flag.
        let is_npc = p.get::<String, _>("info") == "npc";
        match action {
            1 | 2 => transfer_effect(pool, user, victim, is_npc, soft, action).await?,
            3 => delete_effect(pool, user, victim, is_npc, soft).await?,
            4 => hide_effect(pool, user, victim, is_npc, soft).await?,
            5 => seek_effect(pool, user, victim, is_npc, soft).await?,
            13 => install_effect(pool, user, victim, is_npc, soft).await?,
            14 => uninstall_effect(pool, user, victim, is_npc, soft).await?,
            8 => edit_log_effect(pool, pid, victim, is_npc).await?,
            11 => hack_effect(pool, user, victim, is_npc).await?,
            15 => scan_effect(pool, user, victim, is_npc).await?,
            16 => exploit_effect(pool, user, victim, is_npc).await?,
            17 => research_effect(pool, user, soft).await?,
            18 => park_effect(pool, user, soft).await?,
            19 => unpark_effect(pool, user, soft).await?,
            20 => discard_effect(pool, user, soft).await?,
            23 => analyze_effect(pool, user, victim, is_npc, soft).await?,
            25 => reset_ip_effect(pool, user).await?,
            26 => reset_password_effect(pool, user).await?,
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
    sqlx::query("UPDATE log SET text=to_char(now(),'YYYY-MM-DD HH24:MI')||' - '||$3||chr(10)||text WHERE user_id=$1 AND is_npc=$2")
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
///
/// There is no case 9: wiping a log is this same case with an empty text, which is why
/// D_LOG needs nothing of its own.
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
    // deleteTmpLog: the pending text has been spent, and leaving it behind would pile up
    // a row per edit forever.
    sqlx::query("DELETE FROM log_edit WHERE process_id=$1")
        .bind(pid)
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

/// Cases 1 and 2 of completeProcess, download and upload. Both copy a file between two
/// machines, so they differ only in which end is the source.
///
/// The legacy checked here rather than at the start: a copy that no longer fits, or that
/// duplicates something already held, is dropped silently at this point.
async fn transfer_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
    action: i16,
) -> Result<(), sqlx::Error> {
    let download = action == 1;
    let (from, from_npc, to, to_npc) = if download {
        (victim, victim_is_npc, user, false)
    } else {
        (user, false, victim, victim_is_npc)
    };
    let Some(f)=sqlx::query("SELECT soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3").bind(soft).bind(from).bind(from_npc).fetch_optional(pool).await? else {
        return Ok(());
    };
    let name: String = f.get("soft_name");
    let version: i32 = f.get("soft_version");
    let size: i32 = f.get("soft_size");
    let ram: i32 = f.get("soft_ram");
    let soft_type: i16 = f.get("soft_type");
    let licensed: i64 = f.get("licensed_to");
    // "SELECT id FROM software WHERE ... softType AND softVersion AND softName": the same
    // file twice is not worth the disk.
    let held:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software WHERE user_id=$1 AND is_npc=$2 AND soft_type=$3 AND soft_version=$4 AND soft_name=$5 AND NOT soft_hidden)").bind(to).bind(to_npc).bind(soft_type).bind(version).bind(&name).fetch_one(pool).await?;
    if held {
        return Ok(());
    }
    // calculateHDDUsage: total >= used + size, or the file does not land.
    let total:Option<f32>=sqlx::query_scalar("SELECT sum(hdd)::real FROM hardware WHERE user_id=$1 AND is_npc=$2").bind(to).bind(to_npc).fetch_one(pool).await?;
    let used:Option<i64>=sqlx::query_scalar("SELECT sum(soft_size)::bigint FROM software WHERE user_id=$1 AND is_npc=$2").bind(to).bind(to_npc).fetch_one(pool).await?;
    if (total.unwrap_or(0.0) as i64) < used.unwrap_or(0) + size as i64 {
        return Ok(());
    }
    // originalFrom keeps the trail back to the file this copy was taken from.
    sqlx::query("INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,is_npc,original_from,licensed_to) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)")
        .bind(to).bind(&name).bind(version).bind(size).bind(ram).bind(soft_type).bind(to_npc).bind(soft).bind(licensed).execute(pool).await?;
    let verb = if download { "downloaded" } else { "uploaded" };
    log_pair(pool, user, victim, victim_is_npc, verb, &format!("{name}{version}")).await?;
    Ok(())
}

/// Case 3 of completeProcess. A remote delete takes the file away for good.
async fn delete_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let Some(f)=sqlx::query("SELECT soft_name,soft_version FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3").bind(soft).bind(victim).bind(victim_is_npc).fetch_optional(pool).await? else {
        return Ok(());
    };
    let label = format!(
        "{}{}",
        f.get::<String, _>("soft_name"),
        f.get::<i32, _>("soft_version")
    );
    sqlx::query("DELETE FROM software WHERE id=$1")
        .bind(soft)
        .execute(pool)
        .await?;
    log_pair(pool, user, victim, victim_is_npc, "deleted", &label).await?;
    Ok(())
}

/// The two-sided log line every file action in completeProcess wrote: the victim reads it
/// as coming from an address, the attacker as going out to one.
async fn log_pair(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    verb: &str,
    file: &str,
) -> Result<(), sqlx::Error> {
    let attacker_ip:Option<i64>=sqlx::query_scalar("SELECT game_ip FROM users WHERE id=$1").bind(user).fetch_one(pool).await?;
    let Some((victim_ip, _)) = victim_ip(pool, victim, victim_is_npc).await? else {
        return Ok(());
    };
    if let Some(attacker_ip) = attacker_ip {
        add_log(
            pool,
            victim,
            victim_is_npc,
            &format!("[{}] {verb} file {file} at localhost", dotted(attacker_ip)),
        )
        .await?;
        add_log(
            pool,
            user,
            false,
            &format!("localhost {verb} file {file} at [{}]", dotted(victim_ip)),
        )
        .await?;
    }
    Ok(())
}

/// isVirus: the five types the legacy would not let you uninstall, because a virus is
/// meant to be dug out with an antivirus rather than clicked away.
fn is_virus(soft_type: i16) -> bool {
    matches!(soft_type, 8 | 9 | 10 | 20 | 29)
}

/// isExecutable. A virus, a text file, a folder and a torrent are the things that have
/// nothing to run.
fn is_executable(soft_type: i16) -> bool {
    !is_virus(soft_type) && !matches!(soft_type, 17 | 30 | 31)
}

/// Which machine a software process acts on. victim_id 0 means the player's own, which is
/// how the legacy told a local action from a remote one.
fn target_of(user: i64, victim: i64, victim_is_npc: bool) -> (i64, bool) {
    if victim == 0 {
        (user, false)
    } else {
        (victim, victim_is_npc)
    }
}

/// Case 4 of completeProcess. A file is buried under the version of the best hider the
/// player has running; SEEK later has to beat that number to dig it out.
async fn hide_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let (owner, owner_is_npc) = target_of(user, victim, victim_is_npc);
    let Some(f)=sqlx::query("SELECT soft_type FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3").bind(soft).bind(owner).bind(owner_is_npc).fetch_optional(pool).await? else {
        return Ok(());
    };
    // "You cant hide a folder", and a running program cannot be hidden either.
    let soft_type: i16 = f.get("soft_type");
    if soft_type == 31 || soft_type == 29 {
        return Ok(());
    }
    let installed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software_running WHERE soft_id=$1)").bind(soft).fetch_one(pool).await?;
    if installed {
        return Ok(());
    }
    let hider:Option<i32>=sqlx::query_scalar("SELECT max(s.soft_version) FROM software s JOIN software_running r ON r.soft_id=s.id WHERE s.user_id=$1 AND NOT s.is_npc AND s.soft_type=5").bind(user).fetch_one(pool).await?;
    let Some(hider) = hider else {
        return Ok(());
    };
    sqlx::query("UPDATE software SET soft_hidden=TRUE,soft_hidden_with=$2 WHERE id=$1")
        .bind(soft)
        .bind(hider)
        .execute(pool)
        .await?;
    Ok(())
}

/// Case 5 of completeProcess. The seeker has to be at least as good as the hider that
/// buried the file, which is the whole point of keeping soft_hidden_with around.
async fn seek_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let (owner, owner_is_npc) = target_of(user, victim, victim_is_npc);
    let seeker:Option<i32>=sqlx::query_scalar("SELECT max(s.soft_version) FROM software s JOIN software_running r ON r.soft_id=s.id WHERE s.user_id=$1 AND NOT s.is_npc AND s.soft_type=6").bind(user).fetch_one(pool).await?;
    let Some(seeker) = seeker else {
        return Ok(());
    };
    sqlx::query("UPDATE software SET soft_hidden=FALSE,soft_hidden_with=0 WHERE id=$1 AND user_id=$2 AND is_npc=$3 AND soft_hidden AND soft_hidden_with<=$4")
        .bind(soft).bind(owner).bind(owner_is_npc).bind(seeker).execute(pool).await?;
    Ok(())
}

/// Case 13 of completeProcess. Loading a program is what RAM is for: the legacy refused
/// the install when calculateRamUsage had nothing left to give.
async fn install_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let (owner, owner_is_npc) = target_of(user, victim, victim_is_npc);
    let Some(f)=sqlx::query("SELECT soft_type,soft_ram,soft_hidden FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3").bind(soft).bind(owner).bind(owner_is_npc).fetch_optional(pool).await? else {
        return Ok(());
    };
    let soft_type: i16 = f.get("soft_type");
    let need: i32 = f.get("soft_ram");
    if !is_executable(soft_type) || f.get::<bool, _>("soft_hidden") {
        return Ok(());
    }
    let already:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software_running WHERE soft_id=$1)").bind(soft).fetch_one(pool).await?;
    if already {
        return Ok(());
    }
    // calculateRamUsage: total ram of the machine against what is already loaded on it.
    let total:Option<f32>=sqlx::query_scalar("SELECT sum(ram)::real FROM hardware WHERE user_id=$1 AND is_npc=$2").bind(owner).bind(owner_is_npc).fetch_one(pool).await?;
    let used:Option<i64>=sqlx::query_scalar("SELECT sum(s.soft_ram)::bigint FROM software s JOIN software_running r ON r.soft_id=s.id WHERE r.user_id=$1 AND r.is_npc=$2").bind(owner).bind(owner_is_npc).fetch_one(pool).await?;
    if (total.unwrap_or(0.0) as i64) < used.unwrap_or(0) + need as i64 {
        return Ok(());
    }
    sqlx::query("INSERT INTO software_running(soft_id,user_id,is_npc) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(soft).bind(owner).bind(owner_is_npc).execute(pool).await?;
    Ok(())
}

/// Case 14 of completeProcess. Everything but a virus can be unloaded again.
async fn uninstall_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let (owner, owner_is_npc) = target_of(user, victim, victim_is_npc);
    let Some(f)=sqlx::query("SELECT soft_type FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3").bind(soft).bind(owner).bind(owner_is_npc).fetch_optional(pool).await? else {
        return Ok(());
    };
    if is_virus(f.get::<i16, _>("soft_type")) {
        return Ok(());
    }
    sqlx::query("DELETE FROM software_running WHERE soft_id=$1")
        .bind(soft)
        .execute(pool)
        .await?;
    Ok(())
}

/// Case 25 of completeProcess. A new address is the only way out of an IP that half the
/// server has written down, so the legacy checked the draw against every machine before
/// taking it.
///
/// The legacy also billed this to a bank account, priced off uptime by Player::ip_info.
/// The port has neither the uptime column nor the reset timers that price depends on, so
/// the reset is free here until those exist.
async fn reset_ip_effect(pool: &sqlx::PgPool, user: i64) -> Result<(), sqlx::Error> {
    for _ in 0..16 {
        let candidate:i64=sqlx::query_scalar("SELECT (floor(random()*4294967294)+1)::bigint").fetch_one(pool).await?;
        let taken:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE game_ip=$1) OR EXISTS(SELECT 1 FROM npc WHERE npc_ip=$1)").bind(candidate).fetch_one(pool).await?;
        if taken {
            continue;
        }
        sqlx::query("UPDATE users SET game_ip=$2 WHERE id=$1")
            .bind(user)
            .bind(candidate)
            .execute(pool)
            .await?;
        return Ok(());
    }
    Ok(())
}

/// Case 26 of completeProcess, and the answer to being cracked. Changing the password is
/// not only a new string: the legacy went through every list holding the old one and blanked
/// it to 'unknown', so a stolen credential stops working the moment its owner reacts.
///
/// Entries marked 'exploited' were left alone, because those were not taken by guessing
/// the password in the first place.
async fn reset_password_effect(pool: &sqlx::PgPool, user: i64) -> Result<(), sqlx::Error> {
    let Some(ip) = sqlx::query_scalar::<_, i64>("SELECT game_ip FROM users WHERE id=$1")
        .bind(user)
        .fetch_optional(pool)
        .await?
    else {
        return Ok(());
    };
    sqlx::query("UPDATE users SET game_pass=substr(md5(random()::text),1,8) WHERE id=$1")
        .bind(user)
        .execute(pool)
        .await?;
    sqlx::query("UPDATE compromised_lists SET password='unknown',virus_id=0 WHERE ip=$1 AND password<>'exploited'")
        .bind(ip).execute(pool).await?;
    Ok(())
}

/// Case 17 of completeProcess. Research is the upgrade path that does not need a victim:
/// you improve what you already own rather than steal a better copy. Each run raises the
/// version by one and leaves a row in research_history.
async fn research_effect(
    pool: &sqlx::PgPool,
    user: i64,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let Some(f)=sqlx::query("SELECT soft_name,soft_version,soft_type FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc").bind(soft).bind(user).fetch_optional(pool).await? else {
        return Ok(());
    };
    let version: i32 = f.get::<i32, _>("soft_version") + 1;
    sqlx::query("UPDATE software SET soft_version=$1 WHERE id=$2")
        .bind(version)
        .bind(soft)
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO research_history(user_id,software_id,software_type,new_version,software_name) VALUES($1,$2,$3,$4,$5)")
        .bind(user).bind(soft).bind(f.get::<i16,_>("soft_type")).bind(version).bind(f.get::<String,_>("soft_name")).execute(pool).await?;
    Ok(())
}

/// Case 23 of completeProcess. An analyzer reads a machine's CPU and memory, and how close
/// the reading lands depends on the tool: the legacy gave a v10 fifty percent accuracy, a
/// v20 eighty and a v30 the exact figure.
///
/// It switched on the version being exactly 10, 20 or 30 and killed the request otherwise,
/// which any researched analyzer would trip. Reading it as thresholds keeps the same three
/// tiers without that cliff.
async fn analyze_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let Some(version)=sqlx::query_scalar::<_,i32>("SELECT soft_version FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc AND soft_type=16").bind(soft).bind(user).fetch_optional(pool).await? else {
        return Ok(());
    };
    let accuracy = if version >= 30 {
        1.0
    } else if version >= 20 {
        0.8
    } else {
        0.5
    };
    let Some((ip, _)) = victim_ip(pool, victim, victim_is_npc).await? else {
        return Ok(());
    };
    let Some(hw)=sqlx::query("SELECT sum(cpu)::real AS cpu,sum(ram)::real AS ram FROM hardware WHERE user_id=$1 AND is_npc=$2").bind(victim).bind(victim_is_npc).fetch_optional(pool).await? else {
        return Ok(());
    };
    let cpu: f32 = hw.get::<Option<f32>, _>("cpu").unwrap_or(0.0);
    let ram: f32 = hw.get::<Option<f32>, _>("ram").unwrap_or(0.0);
    // The legacy nudged each bound inwards by a random fraction of the error band, so two
    // readings of the same machine never matched exactly.
    let jitter:f32=sqlx::query_scalar::<_,f32>("SELECT (5+random()*45)::real").fetch_one(pool).await?;
    let span_cpu = cpu * (1.0 - accuracy);
    let span_ram = ram * (1.0 - accuracy);
    let cpu_min = (cpu - span_cpu + span_cpu / jitter).round() as i32;
    let cpu_max = (cpu + span_cpu - span_cpu / jitter).round() as i32;
    let ram_min = (ram - span_ram + span_ram / jitter).round() as i32;
    let ram_max = (ram + span_ram - span_ram / jitter).round() as i32;
    sqlx::query("INSERT INTO analyze_results(user_id,ip,cpu_min,cpu_max,ram_min,ram_max) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(user_id,ip) DO UPDATE SET cpu_min=EXCLUDED.cpu_min,cpu_max=EXCLUDED.cpu_max,ram_min=EXCLUDED.ram_min,ram_max=EXCLUDED.ram_max,analyzed_at=now()")
        .bind(user).bind(ip).bind(cpu_min).bind(cpu_max).bind(ram_min).bind(ram_max).execute(pool).await?;
    Ok(())
}

/// Best version of a software type a machine has loaded, which is what getBestSoftware
/// asked for whenever it passed installed = 1.
async fn best_running(
    pool: &sqlx::PgPool,
    owner: i64,
    is_npc: bool,
    soft_type: i16,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar("SELECT COALESCE(max(s.soft_version),0) FROM software s JOIN software_running r ON r.soft_id=s.id WHERE s.user_id=$1 AND s.is_npc=$2 AND s.soft_type=$3 AND NOT s.soft_hidden")
        .bind(owner).bind(is_npc).bind(soft_type).fetch_one(pool).await
}

/// Case 16 of completeProcess, the other way in. Rather than guessing the password you go
/// through a service, so an FTP or SSH exploit is measured against the victim's firewall
/// instead of their hasher.
///
/// Which exploits land decides who you become: ftp alone, ssh alone, or root with both.
/// The credential is filed as 'exploited' rather than a password, and that is exactly why
/// reset_password_effect leaves those rows alone -- there was never a password to change,
/// so changing one takes nothing away.
async fn exploit_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
) -> Result<(), sqlx::Error> {
    let Some((ip, _)) = victim_ip(pool, victim, victim_is_npc).await? else {
        return Ok(());
    };
    let firewall = best_running(pool, victim, victim_is_npc, 4).await?;
    let ftp = best_running(pool, user, false, 13).await?;
    let ssh = best_running(pool, user, false, 14).await?;
    let login = match (ftp > 0 && ftp >= firewall, ssh > 0 && ssh >= firewall) {
        (true, true) => "root",
        (true, false) => "ftp",
        (false, true) => "ssh",
        // Neither exploit clears the firewall, so nothing opens.
        (false, false) => return Ok(()),
    };
    // ALREADY_LISTED: the legacy refused when the machine was already held under a real
    // password, and refused a repeat exploit that would not improve on the user it granted.
    let held:Option<String>=sqlx::query_scalar("SELECT password FROM compromised_lists WHERE user_id=$1 AND ip=$2").bind(user).bind(ip).fetch_optional(pool).await?;
    match held.as_deref() {
        Some("exploited") => {
            sqlx::query("UPDATE compromised_lists SET login=$3,hacked_at=now() WHERE user_id=$1 AND ip=$2")
                .bind(user).bind(ip).bind(login).execute(pool).await?;
        }
        Some(_) => return Ok(()),
        None => {
            sqlx::query("INSERT INTO compromised_lists(user_id,ip,login,password) VALUES($1,$2,$3,'exploited')")
                .bind(user).bind(ip).bind(login).execute(pool).await?;
        }
    }
    sqlx::query("INSERT INTO internet_access(user_id,ip,via_exploit) VALUES($1,$2,true) ON CONFLICT(user_id,ip) DO UPDATE SET via_exploit=true,granted_at=now()")
        .bind(user).bind(ip).execute(pool).await?;
    let cracker = best_running(pool, victim, victim_is_npc, 1).await?;
    sqlx::query("UPDATE users_stats SET exp=exp+$2,hack_count=hack_count+1 WHERE uid=$1")
        .bind(user)
        .bind((firewall + cracker) as i64)
        .execute(pool)
        .await?;
    if let Some(attacker_ip) = sqlx::query_scalar::<_, i64>("SELECT game_ip FROM users WHERE id=$1").bind(user).fetch_optional(pool).await? {
        add_log(pool, victim, victim_is_npc, &format!("[{}] logged in as {login}", dotted(attacker_ip))).await?;
        add_log(pool, user, false, &format!("localhost logged in to [{}] as {login}", dotted(ip))).await?;
    }
    Ok(())
}

/// Case 18 of completeProcess. Parking a file on the external drive, which is the point of
/// having one: it is not on the machine, so formatting or losing the machine leaves it.
///
/// The row keeps the source file's id rather than a fresh one, exactly as the legacy wrote
/// it. A parked file is the same file, somewhere else.
async fn park_effect(pool: &sqlx::PgPool, user: i64, soft: i64) -> Result<(), sqlx::Error> {
    let Some(f)=sqlx::query("SELECT soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc").bind(soft).bind(user).fetch_optional(pool).await? else {
        return Ok(());
    };
    let size: i32 = f.get("soft_size");
    let already:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software_external WHERE id=$1)").bind(soft).fetch_one(pool).await?;
    if already {
        return Ok(());
    }
    // BAD_XHD: the drive's capacity against what is already parked on it.
    let capacity:Option<i64>=sqlx::query_scalar("SELECT sum(size)::bigint FROM hardware_external WHERE user_id=$1").bind(user).fetch_one(pool).await?;
    let used:Option<i64>=sqlx::query_scalar("SELECT sum(soft_size)::bigint FROM software_external WHERE user_id=$1").bind(user).fetch_one(pool).await?;
    if capacity.unwrap_or(0) < used.unwrap_or(0) + size as i64 {
        return Ok(());
    }
    sqlx::query("INSERT INTO software_external(id,user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(soft).bind(user).bind(f.get::<String,_>("soft_name")).bind(f.get::<i32,_>("soft_version")).bind(size).bind(f.get::<i32,_>("soft_ram")).bind(f.get::<i16,_>("soft_type")).bind(f.get::<i64,_>("licensed_to")).execute(pool).await?;
    Ok(())
}

/// Case 19. Bringing a parked file back onto the machine, refused when the disk has no
/// room or a copy is already sitting there.
async fn unpark_effect(pool: &sqlx::PgPool, user: i64, soft: i64) -> Result<(), sqlx::Error> {
    let Some(f)=sqlx::query("SELECT soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to FROM software_external WHERE id=$1 AND user_id=$2").bind(soft).bind(user).fetch_optional(pool).await? else {
        return Ok(());
    };
    let name: String = f.get("soft_name");
    let version: i32 = f.get("soft_version");
    let size: i32 = f.get("soft_size");
    let held:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software WHERE user_id=$1 AND NOT is_npc AND soft_name=$2 AND soft_version=$3)").bind(user).bind(&name).bind(version).fetch_one(pool).await?;
    if held {
        return Ok(());
    }
    // BAD_HDD, the same disk check every copy onto a machine goes through.
    let total:Option<f32>=sqlx::query_scalar("SELECT sum(hdd)::real FROM hardware WHERE user_id=$1 AND NOT is_npc").bind(user).fetch_one(pool).await?;
    let used:Option<i64>=sqlx::query_scalar("SELECT sum(soft_size)::bigint FROM software WHERE user_id=$1 AND NOT is_npc").bind(user).fetch_one(pool).await?;
    if (total.unwrap_or(0.0) as i64) < used.unwrap_or(0) + size as i64 {
        return Ok(());
    }
    sqlx::query("INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(user).bind(&name).bind(version).bind(size).bind(f.get::<i32,_>("soft_ram")).bind(f.get::<i16,_>("soft_type")).bind(f.get::<i64,_>("licensed_to")).execute(pool).await?;
    Ok(())
}

/// Case 20. Clearing a file off the drive, which is the only way to make room on it.
async fn discard_effect(pool: &sqlx::PgPool, user: i64, soft: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM software_external WHERE id=$1 AND user_id=$2")
        .bind(soft)
        .bind(user)
        .execute(pool)
        .await?;
    Ok(())
}
