use rand::{distr::Alphanumeric, Rng};
use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        BankAccountOpenedResponse,
        BankAccountResponse,
        BankTransferRequest,
        BankTransferResponse,
        BitcoinTransferRequest,
        BitcoinTransferResponse,
        CreateBankAccountRequest,
        WalletCreatedResponse,
        WalletResponse,
    },
    repo,
};

//
// ==============================
// BANK ACCOUNTS
// ==============================
//

pub async fn accounts(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<BankAccountResponse>, ApiError> {
    let accounts =
        repo::find_accounts(pool, user_id)
            .await?
            .into_iter()
            .map(BankAccountResponse::from)
            .collect();

    Ok(accounts)
}

/// El PIN del legado eran seis caracteres alfanumericos, ni mas ni menos.
const PIN_LEN: usize = 6;

/// Cuantas veces se reintenta el sorteo del numero de cuenta antes de rendirse.
const ACCOUNT_ATTEMPTS: u8 = 5;

pub async fn open_account(
    pool: &PgPool,
    user_id: i64,
    input: CreateBankAccountRequest,
) -> Result<BankAccountOpenedResponse, ApiError> {
    let valid_pin = input.password.len() == PIN_LEN
        && input
            .password
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric());

    if !(1..=9999).contains(&input.bank_id)
        || !valid_pin
    {
        return Err(ApiError::BadRequest(
            "invalid bank account data".into(),
        ));
    }

    // Una cuenta por banco y jugador.
    let existing =
        repo::count_accounts_at_bank(
            pool,
            user_id,
            input.bank_id,
        )
        .await?;

    if existing > 0 {
        return Err(ApiError::BadRequest(
            "account already exists for this bank"
                .into(),
        ));
    }

    for _ in 0..ACCOUNT_ATTEMPTS {
        let account = rand::rng()
            .random_range(
                111_111_111_i64..=999_999_999,
            );

        let inserted =
            repo::try_insert_account(
                pool,
                account,
                &input.password,
                input.bank_id,
                user_id,
            )
            .await?;

        if inserted == 1 {
            repo::log_account_opened(
                pool,
                user_id,
                &format!(
                    "Bank account {account} opened"
                ),
            )
            .await?;

            return Ok(
                BankAccountOpenedResponse {
                    account,
                    bank_id: input.bank_id,
                    cash: 0,
                },
            );
        }
    }

    Err(ApiError::Internal(
        sqlx::Error::RowNotFound,
    ))
}

//
// ==============================
// BANK TRANSFERS
// ==============================
//

pub async fn transfer(
    pool: &PgPool,
    user_id: i64,
    input: BankTransferRequest,
) -> Result<BankTransferResponse, ApiError> {
    if input.amount <= 0
        || input.from_account == input.to_account
    {
        return Err(ApiError::BadRequest(
            "invalid transfer".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    let source = repo::lock_source_account(
        &mut tx,
        input.from_account,
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    let target = repo::lock_target_account(
        &mut tx,
        input.to_account,
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    // Ser el dueno no basta: hay que traer la clave de la cuenta.
    if source.bank_user != user_id
        || source.bank_pass != input.password
    {
        return Err(ApiError::Unauthorized);
    }

    if source.cash < input.amount {
        return Err(ApiError::BadRequest(
            "insufficient funds".into(),
        ));
    }

    repo::move_cash(
        &mut tx,
        source.id,
        target.id,
        input.amount,
    )
    .await?;

    tx.commit().await?;

    Ok(BankTransferResponse {
        from: input.from_account,
        to: input.to_account,
        amount: input.amount,
    })
}

//
// ==============================
// BITCOIN
// ==============================
//

/// El bitcoin del juego se guarda con siete decimales.
const SATOSHI: f64 = 10_000_000.0;

const ADDRESS_LEN: usize = 34;
const PRIVATE_KEY_LEN: usize = 64;

pub async fn wallets(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<WalletResponse>, ApiError> {
    let wallets =
        repo::find_wallets(pool, user_id)
            .await?
            .into_iter()
            .map(WalletResponse::from)
            .collect();

    Ok(wallets)
}

pub async fn create_wallet(
    pool: &PgPool,
    user_id: i64,
) -> Result<WalletCreatedResponse, ApiError> {
    let address: String = random_string(ADDRESS_LEN);
    let private_key: String =
        random_string(PRIVATE_KEY_LEN);

    repo::insert_wallet(
        pool,
        &address,
        user_id,
        &private_key,
    )
    .await?;

    Ok(WalletCreatedResponse {
        address,
        amount: "0.0000000".to_owned(),
    })
}

pub async fn transfer_bitcoin(
    pool: &PgPool,
    user_id: i64,
    input: BitcoinTransferRequest,
) -> Result<BitcoinTransferResponse, ApiError> {
    let amount = parse_amount(&input.amount)?;
    let canonical = format_amount(amount);

    if input.from == input.to {
        return Err(ApiError::BadRequest(
            "same bitcoin wallet".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    let source =
        repo::lock_wallet(&mut tx, &input.from)
            .await?
            .ok_or(ApiError::NotFound)?;

    if !repo::wallet_exists(&mut tx, &input.to)
        .await?
    {
        return Err(ApiError::NotFound);
    }

    if source.user_id != user_id {
        return Err(ApiError::Unauthorized);
    }

    let available = parse_amount(&source.amount)
        .unwrap_or(0);

    if available < amount {
        return Err(ApiError::BadRequest(
            "insufficient bitcoin funds".into(),
        ));
    }

    repo::move_bitcoin(
        &mut tx,
        &input.from,
        &input.to,
        &canonical,
    )
    .await?;

    tx.commit().await?;

    Ok(BitcoinTransferResponse {
        from: input.from,
        to: input.to,
        amount: canonical,
    })
}

//
// ==============================
// AMOUNTS
// ==============================
//

/// Las cantidades se manejan en enteros de siete decimales para no arrastrar errores de
/// coma flotante entre monederos.
fn parse_amount(
    raw: &str,
) -> Result<i64, ApiError> {
    raw.parse::<f64>()
        .ok()
        .filter(|value| {
            value.is_finite() && *value >= 0.0
        })
        .map(|value| {
            (value * SATOSHI).round() as i64
        })
        .ok_or_else(|| {
            ApiError::BadRequest(
                "invalid bitcoin amount".into(),
            )
        })
}

fn format_amount(amount: i64) -> String {
    format!(
        "{}.{:07}",
        amount / SATOSHI as i64,
        amount % SATOSHI as i64
    )
}

fn random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
