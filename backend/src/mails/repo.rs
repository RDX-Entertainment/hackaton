use sqlx::PgPool;

use super::models::{MailDetailRow, MailRow};

//
// ==============================
// INBOX
// ==============================
//

pub async fn find_for_user(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<MailRow>, sqlx::Error> {
    sqlx::query_as::<_, MailRow>(
        r#"
        SELECT
            id,
            sender_id,
            recipient_id,
            type AS mail_type,
            subject,
            text,
            is_read
        FROM mails
        WHERE (sender_id = $1 OR recipient_id = $1)
          AND NOT is_deleted
        ORDER BY date_sent DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

//
// ==============================
// READ
// ==============================
//

pub async fn find_one(
    pool: &PgPool,
    mail_id: i64,
    user_id: i64,
) -> Result<Option<MailDetailRow>, sqlx::Error> {
    sqlx::query_as::<_, MailDetailRow>(
        r#"
        SELECT
            id,
            sender_id,
            recipient_id,
            type AS mail_type,
            subject,
            text,
            is_read,
            is_deleted
        FROM mails
        WHERE id = $1
          AND (sender_id = $2 OR recipient_id = $2)
        "#,
    )
    .bind(mail_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn mark_read(
    pool: &PgPool,
    mail_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE mails
        SET is_read = true
        WHERE id = $1
        "#,
    )
    .bind(mail_id)
    .execute(pool)
    .await?;

    Ok(())
}

//
// ==============================
// COMPOSE
// ==============================
//

/// El destinatario se escribe por id o por nombre, asi que se resuelven los dos.
pub async fn find_recipient_by_id(
    pool: &PgPool,
    id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT id
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_recipient_by_login(
    pool: &PgPool,
    login: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT id
        FROM users
        WHERE login = $1
        "#,
    )
    .bind(login)
    .fetch_optional(pool)
    .await
}

pub async fn insert(
    pool: &PgPool,
    sender_id: i64,
    recipient_id: i64,
    subject: &str,
    text: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO mails(
            sender_id,
            recipient_id,
            subject,
            text
        )
        VALUES($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(sender_id)
    .bind(recipient_id)
    .bind(subject)
    .bind(text)
    .fetch_one(pool)
    .await
}

//
// ==============================
// DELETE
// ==============================
//

/// Borrar es marcar: el remitente conserva su copia.
pub async fn soft_delete(
    pool: &PgPool,
    mail_id: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE mails
        SET is_deleted = true
        WHERE id = $1
          AND recipient_id = $2
          AND NOT is_deleted
        "#,
    )
    .bind(mail_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
