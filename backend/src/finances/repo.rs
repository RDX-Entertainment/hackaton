use sqlx::{PgPool, Postgres, Transaction};

use super::models::{
    BankAccountLockRow,
    BankAccountRow,
    BankAccountTargetRow,
    WalletLockRow,
    WalletRow,
};

//
// ==============================
// BANK ACCOUNTS
// ==============================
//

pub async fn find_accounts(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<BankAccountRow>, sqlx::Error> {
    sqlx::query_as::<_, BankAccountRow>(
        r#"
        SELECT
            id,
            bank_acc,
            bank_id,
            cash
        FROM bank_accounts
        WHERE bank_user = $1
        ORDER BY date_created DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn count_accounts_at_bank(
    pool: &PgPool,
    user_id: i64,
    bank_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM bank_accounts
        WHERE bank_user = $1
          AND bank_id = $2
        "#,
    )
    .bind(user_id)
    .bind(bank_id)
    .fetch_one(pool)
    .await
}

/// El numero de cuenta se sortea, asi que puede chocar con uno ya dado: el ON CONFLICT
/// deja que quien llama vuelva a intentarlo en vez de reventar.
pub async fn try_insert_account(
    pool: &PgPool,
    account: i64,
    password: &str,
    bank_id: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO bank_accounts(
            bank_acc,
            bank_pass,
            bank_id,
            bank_user
        )
        VALUES($1, $2, $3, $4)
        ON CONFLICT(bank_acc) DO NOTHING
        "#,
    )
    .bind(account)
    .bind(password)
    .bind(bank_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn log_account_opened(
    pool: &PgPool,
    user_id: i64,
    text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO log(user_id, text)
        VALUES($1, $2)
        "#,
    )
    .bind(user_id)
    .bind(text)
    .execute(pool)
    .await?;

    Ok(())
}

//
// ==============================
// BANK TRANSFERS
// ==============================
//

pub async fn lock_source_account(
    tx: &mut Transaction<'_, Postgres>,
    account: i64,
) -> Result<Option<BankAccountLockRow>, sqlx::Error>
{
    sqlx::query_as::<_, BankAccountLockRow>(
        r#"
        SELECT
            id,
            bank_pass,
            bank_user,
            cash
        FROM bank_accounts
        WHERE bank_acc = $1
        FOR UPDATE
        "#,
    )
    .bind(account)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn lock_target_account(
    tx: &mut Transaction<'_, Postgres>,
    account: i64,
) -> Result<
    Option<BankAccountTargetRow>,
    sqlx::Error,
> {
    sqlx::query_as::<_, BankAccountTargetRow>(
        r#"
        SELECT
            id,
            bank_user
        FROM bank_accounts
        WHERE bank_acc = $1
        FOR UPDATE
        "#,
    )
    .bind(account)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn move_cash(
    tx: &mut Transaction<'_, Postgres>,
    from_id: i64,
    to_id: i64,
    amount: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE bank_accounts
        SET cash = cash - $1
        WHERE id = $2
        "#,
    )
    .bind(amount)
    .bind(from_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE bank_accounts
        SET cash = cash + $1
        WHERE id = $2
        "#,
    )
    .bind(amount)
    .bind(to_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// BITCOIN
// ==============================
//

pub async fn find_wallets(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<WalletRow>, sqlx::Error> {
    sqlx::query_as::<_, WalletRow>(
        r#"
        SELECT
            address,
            amount::text AS amount,
            created_at
        FROM bitcoin_wallets
        WHERE user_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn insert_wallet(
    pool: &PgPool,
    address: &str,
    user_id: i64,
    private_key: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO bitcoin_wallets(
            address,
            user_id,
            private_key
        )
        VALUES($1, $2, $3)
        "#,
    )
    .bind(address)
    .bind(user_id)
    .bind(private_key)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn lock_wallet(
    tx: &mut Transaction<'_, Postgres>,
    address: &str,
) -> Result<Option<WalletLockRow>, sqlx::Error> {
    sqlx::query_as::<_, WalletLockRow>(
        r#"
        SELECT
            address,
            user_id,
            amount::text AS amount
        FROM bitcoin_wallets
        WHERE address = $1
        FOR UPDATE
        "#,
    )
    .bind(address)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn wallet_exists(
    tx: &mut Transaction<'_, Postgres>,
    address: &str,
) -> Result<bool, sqlx::Error> {
    let found: Option<String> =
        sqlx::query_scalar(
            r#"
        SELECT address
        FROM bitcoin_wallets
        WHERE address = $1
        FOR UPDATE
        "#,
        )
        .bind(address)
        .fetch_optional(&mut **tx)
        .await?;

    Ok(found.is_some())
}

/// La cantidad viaja como texto y entra en la columna numeric por CAST, que es lo que
/// evita perder decimales por el camino.
pub async fn move_bitcoin(
    tx: &mut Transaction<'_, Postgres>,
    from: &str,
    to: &str,
    amount: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE bitcoin_wallets
        SET amount = amount - CAST($1 AS numeric)
        WHERE address = $2
        "#,
    )
    .bind(amount)
    .bind(from)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE bitcoin_wallets
        SET amount = amount + CAST($1 AS numeric)
        WHERE address = $2
        "#,
    )
    .bind(amount)
    .bind(to)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
