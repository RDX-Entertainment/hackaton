use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct MailRow {
    pub id: i64,
    pub sender_id: i64,
    pub recipient_id: i64,
    pub mail_type: i16,
    pub subject: String,
    pub text: String,
    pub is_read: bool,
}

/// La fila de detalle lleva ademas is_deleted, que decide si el destinatario todavia
/// puede verlo.
#[derive(Debug, Clone, FromRow)]
pub struct MailDetailRow {
    pub id: i64,
    pub sender_id: i64,
    pub recipient_id: i64,
    pub mail_type: i16,
    pub subject: String,
    pub text: String,
    pub is_read: bool,
    pub is_deleted: bool,
}
