use chrono::{DateTime, Utc};
use sqlx::FromRow;

//
// ==============================
// BANK
// ==============================
//

#[derive(Debug, Clone, FromRow)]
pub struct BankAccountRow {
    pub id: i64,
    pub bank_acc: i64,
    pub bank_id: i64,
    pub cash: i64,
}

/// La fila que se bloquea para transferir. Lleva la clave y el dueno porque los dos se
/// comprueban antes de mover un centimo.
#[derive(Debug, Clone, FromRow)]
pub struct BankAccountLockRow {
    pub id: i64,
    pub bank_pass: String,
    pub bank_user: i64,
    pub cash: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct BankAccountTargetRow {
    pub id: i64,
    pub bank_user: i64,
}

//
// ==============================
// BITCOIN
// ==============================
//

#[derive(Debug, Clone, FromRow)]
pub struct WalletRow {
    pub address: String,
    pub amount: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct WalletLockRow {
    pub address: String,
    pub user_id: i64,
    pub amount: String,
}
