use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::models::{BankAccountRow, BankRow, ChargeRow, TransactionRow, WalletRow};

//
// ==============================
// BANK ACCOUNTS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct BankAccountResponse {
    pub id: i64,
    pub account: i64,
    pub bank_id: i64,
    pub bank_name: String,
    pub cash: i64,
    pub pin_set: bool,
}

impl From<BankAccountRow> for BankAccountResponse {
    fn from(value: BankAccountRow) -> Self {
        Self {
            id: value.id,
            account: value.bank_acc,
            bank_id: value.bank_id,
            bank_name: value.bank_name,
            cash: value.cash,
            pin_set: value.pin_set,
        }
    }
}

/// Los bancos donde se puede abrir cuenta: organizaciones con el comportamiento `banking`.
#[derive(Debug, Serialize)]
pub struct BankResponse {
    pub id: i64,
    pub slug: String,
    pub name: String,
}

impl From<BankRow> for BankResponse {
    fn from(value: BankRow) -> Self {
        Self {
            id: value.id,
            slug: value.slug,
            name: value.name,
        }
    }
}

/// El PIN que el titular elige la primera vez. No sirve para cambiarlo despues: una cuenta
/// que ya tiene PIN no pasa por aqui.
#[derive(Debug, Deserialize)]
pub struct SetPinRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBankAccountRequest {
    pub bank_id: i64,
    pub password: String,
}

/// Cuanto se mueve, en positivo o en negativo. Es el panel quien lo pide, asi que aqui
/// «ingresar» y «retirar» son el mismo gesto con el signo cambiado.
#[derive(Debug, Deserialize)]
pub struct AdjustCashRequest {
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct BankAccountAdjustedResponse {
    pub id: i64,
    pub cash: i64,
}

#[derive(Debug, Serialize)]
pub struct BankAccountOpenedResponse {
    pub account: i64,
    pub bank_id: i64,
    pub cash: i64,
}

//
// ==============================
// BANK TRANSFERS
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct BankTransferRequest {
    pub from_account: i64,
    pub password: String,
    pub to_account: i64,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct BankTransferResponse {
    pub from: i64,
    pub to: i64,
    pub amount: i64,
}

//
// ==============================
// BITCOIN
// ==============================
//

/// `amount` en satoshis. Es el cliente quien decide donde va la coma.
#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub address: String,
    pub amount: i64,
    pub created_at: DateTime<Utc>,
}

impl From<WalletRow> for WalletResponse {
    fn from(value: WalletRow) -> Self {
        Self {
            address: value.address,
            amount: value.amount,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WalletCreatedResponse {
    pub address: String,
    pub amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct BitcoinTransferRequest {
    pub from: String,
    pub to: String,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct BitcoinTransferResponse {
    pub from: String,
    pub to: String,
    pub amount: i64,
}

//
// ==============================
// OPERACIONES DE BANCO
// ==============================
//

/// Lo que se ve al entrar en una cuenta con su PIN. Nunca incluye el PIN.
#[derive(Debug, Serialize)]
pub struct AccountAccessResponse {
    pub account: i64,
    pub bank_id: i64,
    pub bank_name: String,
    pub holder: String,
    pub cash: i64,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateRequest {
    pub account: i64,
    pub pin: String,
}

#[derive(Debug, Deserialize)]
pub struct BankTransferOrder {
    pub account: i64,
    pub pin: String,
    pub to_account: i64,
    pub amount: i64,
    #[serde(default)]
    pub concept: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateChargeRequest {
    /// La cuenta que cobra.
    pub account: i64,
    pub pin: String,
    /// A quien se le cobra. Sin esto el cobro queda abierto y lo paga cualquiera.
    #[serde(default)]
    pub from_account: Option<i64>,
    pub amount: i64,
    pub concept: String,
}

#[derive(Debug, Deserialize)]
pub struct PayChargeRequest {
    pub account: i64,
    pub pin: String,
}

#[derive(Debug, Serialize)]
pub struct ChargeResponse {
    pub id: i64,
    pub to_account: i64,
    pub from_account: Option<i64>,
    pub amount: i64,
    pub concept: String,
    pub status: String,
    pub paid_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    /// Un cobro sin destinatario lo paga quien lo tenga delante.
    pub open: bool,
}

impl From<ChargeRow> for ChargeResponse {
    fn from(value: ChargeRow) -> Self {
        Self {
            id: value.id,
            to_account: value.to_account,
            from_account: value.from_account,
            amount: value.amount,
            concept: value.concept,
            status: value.status,
            paid_by: value.paid_by,
            created_at: value.created_at,
            paid_at: value.paid_at,
            open: value.from_account.is_none(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub account: i64,
    pub other_account: Option<i64>,
    /// Positivo entra, negativo sale.
    pub amount: i64,
    pub balance: i64,
    pub concept: String,
}

impl From<TransactionRow> for TransactionResponse {
    fn from(value: TransactionRow) -> Self {
        Self {
            account: value.account,
            other_account: value.other_account,
            amount: value.amount,
            balance: value.balance,
            concept: value.concept,
        }
    }
}
