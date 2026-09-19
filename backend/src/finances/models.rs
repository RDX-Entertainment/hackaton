use chrono::{DateTime, Utc};
use sqlx::FromRow;

//
// ==============================
// BANK
// ==============================
//

#[derive(Debug, Clone, FromRow)]
pub struct BankAccountRow {
    pub is_main: bool,
    pub id: i64,
    pub bank_acc: i64,
    pub bank_id: i64,
    pub bank_name: String,
    pub cash: i64,
    /// Si el titular ya eligio PIN. El PIN en si no sale nunca de la base.
    pub pin_set: bool,
}

/// Una organizacion con el comportamiento `banking`. Es lo que el jugador elige al abrir
/// una cuenta, en vez del numero suelto que se tecleaba antes.
#[derive(Debug, Clone, FromRow)]
pub struct BankRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
}

/// La fila que se bloquea para transferir. Lleva la clave y el dueno porque los dos se
/// comprueban antes de mover un centimo. El titular es un jugador o una organizacion, nunca
/// los dos: por eso las dos columnas son opcionales y la base obliga a que haya exactamente
/// una.
#[derive(Debug, Clone, FromRow)]
pub struct BankAccountLockRow {
    pub id: i64,
    /// `None` mientras el titular no haya elegido PIN: la cuenta existe y no se puede usar.
    pub bank_pass: Option<String>,
    pub bank_user: Option<i64>,
    pub cash: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct BankAccountTargetRow {
    pub id: i64,
}

//
// ==============================
// BITCOIN
// ==============================
//

/// `amount` viaja en satoshis: siete decimales guardados como entero. Quien pone la coma
/// es el cliente.
#[derive(Debug, Clone, FromRow)]
pub struct WalletRow {
    pub address: String,
    pub amount: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct WalletLockRow {
    pub user_id: i64,
    pub amount: i64,
}

//
// ==============================
// OPERACIONES DE BANCO
// ==============================
//

/// Una cuenta vista por su numero, con lo justo para autenticar y operar. El PIN entra en el
/// modelo porque hay que compararlo, y no sale nunca en un DTO.
#[derive(Debug, Clone, FromRow)]
pub struct AccountAuthRow {
    pub bank_user: Option<i64>,
    pub is_main: bool,
    pub number: i64,
    pub bank_id: i64,
    pub bank_name: String,
    pub holder: String,
    pub pin: Option<String>,
    pub cash: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct ChargeRow {
    pub id: i64,
    pub to_account: i64,
    pub from_account: Option<i64>,
    pub amount: i64,
    pub concept: String,
    pub status: String,
    pub paid_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TransactionRow {
    pub account: i64,
    pub other_account: Option<i64>,
    pub amount: i64,
    pub balance: i64,
    pub concept: String,
    pub origin_ip: Option<i64>,
    pub created_at: DateTime<Utc>,
}
