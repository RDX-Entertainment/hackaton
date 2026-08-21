use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::models::{BankAccountRow, WalletRow};

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
    pub cash: i64,
}

impl From<BankAccountRow> for BankAccountResponse {
    fn from(value: BankAccountRow) -> Self {
        Self {
            id: value.id,
            account: value.bank_acc,
            bank_id: value.bank_id,
            cash: value.cash,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateBankAccountRequest {
    pub bank_id: i64,
    pub password: String,
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

#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub address: String,
    pub amount: String,
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
    pub amount: String,
}

#[derive(Debug, Deserialize)]
pub struct BitcoinTransferRequest {
    pub from: String,
    pub to: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct BitcoinTransferResponse {
    pub from: String,
    pub to: String,
    pub amount: String,
}
