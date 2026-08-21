use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        ComposeMailRequest,
        MailResponse,
        MailSentResponse,
    },
    repo,
};

//
// ==============================
// INBOX
// ==============================
//

pub async fn inbox(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<MailResponse>, ApiError> {
    let mails =
        repo::find_for_user(pool, user_id)
            .await?
            .into_iter()
            .map(MailResponse::from)
            .collect();

    Ok(mails)
}

//
// ==============================
// READ
// ==============================
//

/// Abrir un correo lo marca leido, pero solo para quien lo recibio: el remitente
/// releyendo lo suyo no cambia nada.
pub async fn read(
    pool: &PgPool,
    user_id: i64,
    mail_id: i64,
) -> Result<MailResponse, ApiError> {
    let mail =
        repo::find_one(pool, mail_id, user_id)
            .await?
            .ok_or(ApiError::NotFound)?;

    // Un correo borrado sigue siendo visible para quien lo envio.
    if mail.is_deleted
        && mail.sender_id != user_id
    {
        return Err(ApiError::NotFound);
    }

    let mut response =
        MailResponse::from(mail.clone());

    if mail.recipient_id == user_id {
        repo::mark_read(pool, mail_id).await?;
        response.read = true;
    }

    Ok(response)
}

//
// ==============================
// COMPOSE
// ==============================
//

/// El asunto del legado cabia en cuarenta caracteres.
const SUBJECT_MAX: usize = 40;

pub async fn compose(
    pool: &PgPool,
    user_id: i64,
    input: ComposeMailRequest,
) -> Result<MailSentResponse, ApiError> {
    let subject = input.subject.trim();
    let text = input.text.trim();

    if subject.is_empty()
        || subject.len() > SUBJECT_MAX
        || text.is_empty()
    {
        return Err(ApiError::BadRequest(
            "invalid mail".into(),
        ));
    }

    let recipient = resolve_recipient(
        pool,
        input.to.trim(),
    )
    .await?;

    if recipient == user_id {
        return Err(ApiError::BadRequest(
            "you can not send email to yourself"
                .into(),
        ));
    }

    let id = repo::insert(
        pool,
        user_id,
        recipient,
        subject,
        text,
    )
    .await?;

    Ok(MailSentResponse {
        id,
        to: recipient,
    })
}

/// El destinatario se escribe por id o por nombre de jugador.
async fn resolve_recipient(
    pool: &PgPool,
    to: &str,
) -> Result<i64, ApiError> {
    let found = match to.parse::<i64>() {
        Ok(id) => {
            repo::find_recipient_by_id(pool, id)
                .await?
        }
        Err(_) => {
            repo::find_recipient_by_login(
                pool, to,
            )
            .await?
        }
    };

    found.ok_or(ApiError::NotFound)
}

//
// ==============================
// DELETE
// ==============================
//

pub async fn delete(
    pool: &PgPool,
    user_id: i64,
    mail_id: i64,
) -> Result<(), ApiError> {
    let affected = repo::soft_delete(
        pool,
        mail_id,
        user_id,
    )
    .await?;

    if affected == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(())
}
