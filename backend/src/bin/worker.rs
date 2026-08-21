use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
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
    // Completion marker for the legacy Process::completeProcess polling flow.
    sqlx::query("UPDATE processes SET time_worked=time_ideal,completed_at=now() WHERE completed_at IS NULL AND NOT is_paused AND time_end<=now()")
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
