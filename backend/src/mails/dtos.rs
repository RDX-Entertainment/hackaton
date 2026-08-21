use serde::{Deserialize, Serialize};

use super::models::{MailDetailRow, MailRow};

//
// ==============================
// INBOX
// ==============================
//

#[derive(Debug, Serialize)]
pub struct MailResponse {
    pub id: i64,
    pub from: i64,
    pub to: i64,

    #[serde(rename = "type")]
    pub mail_type: i16,

    pub subject: String,
    pub text: String,
    pub read: bool,
}

impl From<MailRow> for MailResponse {
    fn from(value: MailRow) -> Self {
        Self {
            id: value.id,
            from: value.sender_id,
            to: value.recipient_id,
            mail_type: value.mail_type,
            subject: value.subject,
            text: value.text,
            read: value.is_read,
        }
    }
}

impl From<MailDetailRow> for MailResponse {
    fn from(value: MailDetailRow) -> Self {
        Self {
            id: value.id,
            from: value.sender_id,
            to: value.recipient_id,
            mail_type: value.mail_type,
            subject: value.subject,
            text: value.text,
            read: value.is_read,
        }
    }
}

//
// ==============================
// COMPOSE
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct ComposeMailRequest {
    pub to: String,
    pub subject: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct MailSentResponse {
    pub id: i64,
    pub to: i64,
}
