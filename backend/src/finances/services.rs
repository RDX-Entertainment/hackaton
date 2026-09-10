use rand::{distr::Alphanumeric, Rng};
use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        BankAccountAdjustedResponse,
        BankAccountOpenedResponse,
        BankAccountResponse,
        BankResponse,
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

pub async fn banks(
    pool: &PgPool,
) -> Result<Vec<BankResponse>, ApiError> {
    let banks = repo::find_banks(pool)
        .await?
        .into_iter()
        .map(BankResponse::from)
        .collect();

    Ok(banks)
}

/// El PIN del legado eran seis caracteres alfanumericos, ni mas ni menos.
const PIN_LEN: usize = 6;

fn valid_pin(password: &str) -> bool {
    password.len() == PIN_LEN
        && password
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric())
}

/// El PIN que el jugador elige la primera vez que entra en Finanzas.
///
/// Una cuenta que ya tiene PIN no pasa por aqui: esto no es «cambiar el PIN», que exigiria
/// traer el actual. Que la cuenta sea suya y siga sin PIN lo decide el UPDATE, asi que dos
/// peticiones a la vez no pueden pisarse.
pub async fn set_pin(
    pool: &PgPool,
    user_id: i64,
    account_id: i64,
    password: String,
) -> Result<(), ApiError> {
    if !valid_pin(&password) {
        return Err(ApiError::BadRequest(
            "el PIN son 6 caracteres alfanumericos".into(),
        ));
    }

    if repo::set_pin_if_unset(
        pool,
        account_id,
        user_id,
        &password,
    )
    .await?
        == 0
    {
        // No escribir nada tiene dos motivos muy distintos, y al jugador le importan. Si la
        // cuenta es suya, es que ya tenia PIN: decirselo no le descubre nada que no sepa.
        // Si no lo es, la cuenta no existe para el.
        return Err(
            if repo::account_belongs_to(pool, account_id, user_id).await? {
                ApiError::BadRequest(
                    "PIN_ALREADY_SET: esta cuenta ya tiene PIN".into(),
                )
            } else {
                ApiError::NotFound
            },
        );
    }

    Ok(())
}

/// Cuantas veces se reintenta el sorteo del numero de cuenta antes de rendirse.
const ACCOUNT_ATTEMPTS: u8 = 5;

pub async fn open_account(
    pool: &PgPool,
    user_id: i64,
    input: CreateBankAccountRequest,
) -> Result<BankAccountOpenedResponse, ApiError> {
    let opened = open_for(pool, Some(user_id), None, input).await?;

    repo::log_account_opened(
        pool,
        user_id,
        &format!(
            "Bank account {} opened",
            opened.account
        ),
    )
    .await?;

    Ok(opened)
}

/// La clave ajena de la tabla es quien decide si una organizacion es un banco. Cuando dice
/// que no, el jugador merece leer el motivo y no un 500.
fn not_a_bank(error: sqlx::Error) -> ApiError {
    let culpable = match &error {
        sqlx::Error::Database(inner) => inner.constraint(),
        _ => None,
    };

    if culpable == Some("bank_accounts_bank_is_a_bank_fkey") {
        return ApiError::BadRequest(
            "NOT_A_BANK: esa organizacion no ofrece cuentas".into(),
        );
    }

    ApiError::from(error)
}

/// El alta de una cuenta, sea de quien sea. El titular llega como una de las dos opciones
/// y la otra en `None`; que solo haya una lo obliga el CHECK de la tabla.
///
/// El numero se sortea, asi que puede chocar con uno ya dado: por eso los reintentos.
async fn open_for(
    pool: &PgPool,
    user_id: Option<i64>,
    organization_id: Option<i64>,
    input: CreateBankAccountRequest,
) -> Result<BankAccountOpenedResponse, ApiError> {
    if !valid_pin(&input.password) {
        return Err(ApiError::BadRequest(
            "invalid bank account data".into(),
        ));
    }

    // Una cuenta por banco y titular.
    let existing = match (user_id, organization_id) {
        (Some(user), _) => {
            repo::count_accounts_at_bank(
                pool,
                user,
                input.bank_id,
            )
            .await?
        }
        (_, Some(organization)) => {
            repo::count_organization_accounts_at_bank(
                pool,
                organization,
                input.bank_id,
            )
            .await?
        }
        _ => {
            return Err(ApiError::BadRequest(
                "an account needs an owner".into(),
            ))
        }
    };

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

        let inserted = repo::try_insert_account(
            pool,
            account,
            &input.password,
            input.bank_id,
            user_id,
            organization_id,
        )
        .await
        .map_err(not_a_bank)?;

        if inserted == 1 {
            return Ok(BankAccountOpenedResponse {
                account,
                bank_id: input.bank_id,
                cash: 0,
            });
        }
    }

    Err(ApiError::Internal(
        sqlx::Error::RowNotFound,
    ))
}

/// Una organizacion tambien puede tener cuenta: el FBI la tiene en Pinance sin ser un banco.
/// La abre el panel, no el jugador.
pub async fn open_organization_account(
    pool: &PgPool,
    organization_id: i64,
    input: CreateBankAccountRequest,
) -> Result<BankAccountOpenedResponse, ApiError> {
    open_for(pool, None, Some(organization_id), input).await
}

/// El panel tambien abre cuentas a un jugador: sin una cuenta no hay donde ingresarle nada.
pub async fn open_user_account(
    pool: &PgPool,
    user_id: i64,
    input: CreateBankAccountRequest,
) -> Result<BankAccountOpenedResponse, ApiError> {
    open_for(pool, Some(user_id), None, input).await
}

pub async fn user_accounts(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<BankAccountResponse>, ApiError> {
    let accounts = repo::find_user_accounts(pool, user_id)
        .await?
        .into_iter()
        .map(BankAccountResponse::from)
        .collect();

    Ok(accounts)
}

/// Ingresar o retirar desde el panel. Es dinero creado y destruido a mano, igual que el
/// hardware del modo creativo: mientras el juego no tenga de donde salga el dinero, esta es
/// la unica forma de que exista.
///
/// El delta se aplica sobre el saldo bloqueado, no sobre el que vio el navegador, y no puede
/// dejar la cuenta en rojo: una cuenta a cero es un jugador sin fondos, una en negativo es
/// una deuda que el juego no sabe cobrar.
pub async fn adjust_cash(
    pool: &PgPool,
    account_id: i64,
    amount: i64,
) -> Result<BankAccountAdjustedResponse, ApiError> {
    if amount == 0 {
        return Err(ApiError::BadRequest(
            "nothing to move".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    let cash = repo::lock_cash_by_id(&mut tx, account_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let updated = cash.checked_add(amount).ok_or_else(|| {
        ApiError::BadRequest("amount is out of range".into())
    })?;

    if updated < 0 {
        return Err(ApiError::BadRequest(
            "INSUFFICIENT_FUNDS: la cuenta no tiene tanto".into(),
        ));
    }

    repo::set_cash_by_id(&mut tx, account_id, updated).await?;
    tx.commit().await?;

    Ok(BankAccountAdjustedResponse {
        id: account_id,
        cash: updated,
    })
}

pub async fn organization_accounts(
    pool: &PgPool,
    organization_id: i64,
) -> Result<Vec<BankAccountResponse>, ApiError> {
    let accounts =
        repo::find_organization_accounts(
            pool,
            organization_id,
        )
        .await?
        .into_iter()
        .map(BankAccountResponse::from)
        .collect();

    Ok(accounts)
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
    crate::onboarding::services::guard(pool, user_id, "banco.transferir").await?;

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

    // Ser el dueno no basta: hay que traer la clave de la cuenta. Y una cuenta de una
    // organizacion no tiene jugador detras, asi que ningun jugador la mueve desde aqui.
    if source.bank_user != Some(user_id)
        || source.bank_pass.as_deref()
            != Some(input.password.as_str())
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
        amount: 0,
    })
}

/// Las cantidades viajan en satoshis: enteros de siete decimales. Ni aqui ni en la base
/// hay coma flotante, asi que sumar y restar saldos es exacto.
pub async fn transfer_bitcoin(
    pool: &PgPool,
    user_id: i64,
    input: BitcoinTransferRequest,
) -> Result<BitcoinTransferResponse, ApiError> {
    let amount = input.amount;

    if amount <= 0 {
        return Err(ApiError::BadRequest(
            "invalid bitcoin amount".into(),
        ));
    }

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

    if source.amount < amount {
        return Err(ApiError::BadRequest(
            "insufficient bitcoin funds".into(),
        ));
    }

    repo::move_bitcoin(
        &mut tx,
        &input.from,
        &input.to,
        amount,
    )
    .await?;

    tx.commit().await?;

    Ok(BitcoinTransferResponse {
        from: input.from,
        to: input.to,
        amount,
    })
}

fn random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
