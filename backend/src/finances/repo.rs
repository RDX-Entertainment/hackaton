use sqlx::{PgPool, Postgres, Transaction};

use super::models::{
    AccountAuthRow,
    BankAccountLockRow,
    BankAccountRow,
    BankAccountTargetRow,
    BankRow,
    ChargeRow,
    TransactionRow,
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
            a.id,
            a.bank_acc,
            a.bank_id,
            o.name AS bank_name,
            a.cash,
            a.bank_pass IS NOT NULL AS pin_set
        FROM bank_accounts a
        JOIN organizations o
            ON o.id = a.bank_id
        WHERE a.bank_user = $1
        ORDER BY a.date_created DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Donde se puede abrir una cuenta. Un banco es una organizacion activa que lleva el
/// comportamiento `banking`, que es la misma condicion que hace cumplir la clave ajena de
/// `bank_accounts`.
pub async fn find_banks(
    pool: &PgPool,
) -> Result<Vec<BankRow>, sqlx::Error> {
    sqlx::query_as::<_, BankRow>(
        r#"
        SELECT
            o.id,
            o.slug,
            o.name
        FROM organizations o
        JOIN organization_behaviors b
            ON b.organization_id = o.id
           AND b.code = 'banking'
        WHERE o.active
        ORDER BY o.name
        "#,
    )
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
///
/// El titular llega como una de las dos opciones y la otra va en `None`. No se comprueba
/// aqui que solo haya una: lo obliga el CHECK de la tabla, que no se puede saltar.
pub async fn try_insert_account(
    pool: &PgPool,
    account: i64,
    password: &str,
    bank_id: i64,
    user_id: Option<i64>,
    organization_id: Option<i64>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO bank_accounts(
            bank_acc,
            bank_pass,
            bank_id,
            bank_user,
            owner_organization_id
        )
        VALUES($1, $2, $3, $4, $5)
        ON CONFLICT(bank_acc) DO NOTHING
        "#,
    )
    .bind(account)
    .bind(password)
    .bind(bank_id)
    .bind(user_id)
    .bind(organization_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Lo que el panel necesita ver de una cuenta. Las dos consultas de abajo solo se
/// diferencian en de quien son las cuentas, asi que la parte comun se escribe una vez.
/// No lleva la clave: el panel las administra, no las usa para entrar.
const ADMIN_ACCOUNT_QUERY: &str = r#"
    SELECT
        a.id,
        a.bank_acc,
        a.bank_id,
        o.name AS bank_name,
        a.cash,
        a.bank_pass IS NOT NULL AS pin_set
    FROM bank_accounts a
    JOIN organizations o
        ON o.id = a.bank_id
"#;

pub async fn find_organization_accounts(
    pool: &PgPool,
    organization_id: i64,
) -> Result<Vec<BankAccountRow>, sqlx::Error> {
    let query = format!(
        "{ADMIN_ACCOUNT_QUERY} WHERE a.owner_organization_id = $1 ORDER BY a.date_created DESC"
    );

    sqlx::query_as::<_, BankAccountRow>(&query)
        .bind(organization_id)
        .fetch_all(pool)
        .await
}

pub async fn find_user_accounts(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<BankAccountRow>, sqlx::Error> {
    let query = format!(
        "{ADMIN_ACCOUNT_QUERY} WHERE a.bank_user = $1 ORDER BY a.date_created DESC"
    );

    sqlx::query_as::<_, BankAccountRow>(&query)
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn count_user_accounts_at_bank(
    pool: &PgPool,
    user_id: i64,
    bank_id: i64,
) -> Result<i64, sqlx::Error> {
    count_accounts_at_bank(pool, user_id, bank_id).await
}

/// Bloquea la cuenta y devuelve su saldo. El ajuste del panel se decide sobre este numero,
/// no sobre el que vio el navegador hace un rato.
pub async fn lock_cash_by_id(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT cash
        FROM bank_accounts
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Guarda el PIN elegido, y solo si la cuenta es del titular y todavia no tenia uno. Las
/// dos condiciones van en el WHERE y no en un `if` previo: entre comprobar y escribir cabe
/// otra peticion, y aqui el que pierda la carrera no debe pisar el PIN del que gano.
pub async fn set_pin_if_unset(
    pool: &PgPool,
    account_id: i64,
    user_id: i64,
    password: &str,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        UPDATE bank_accounts
        SET bank_pass = $3
        WHERE id = $1
          AND bank_user = $2
          AND bank_pass IS NULL
        "#,
    )
    .bind(account_id)
    .bind(user_id)
    .bind(password)
    .execute(pool)
    .await?;

    Ok(done.rows_affected())
}

/// Si la cuenta es de este jugador. Se usa para contar la verdad cuando el alta del PIN no
/// escribe nada: no es lo mismo «ya tienes PIN» que «esa cuenta no es tuya».
pub async fn account_belongs_to(
    pool: &PgPool,
    account_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM bank_accounts
            WHERE id = $1 AND bank_user = $2
        )
        "#,
    )
    .bind(account_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn set_cash_by_id(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
    cash: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE bank_accounts
        SET cash = $2
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .bind(cash)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn count_organization_accounts_at_bank(
    pool: &PgPool,
    organization_id: i64,
    bank_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM bank_accounts
        WHERE owner_organization_id = $1
          AND bank_id = $2
        "#,
    )
    .bind(organization_id)
    .bind(bank_id)
    .fetch_one(pool)
    .await
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
            id
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
            amount,
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
            user_id,
            amount
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
    amount: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE bitcoin_wallets
        SET amount = amount - $1
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
        SET amount = amount + $1
        WHERE address = $2
        "#,
    )
    .bind(amount)
    .bind(to)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// OPERACIONES DE BANCO
// ==============================
//

/// El titular en una sola columna. Una cuenta es de un jugador o de una organizacion, nunca
/// de los dos, asi que el COALESCE nunca tiene que elegir entre dos nombres.
const ACCOUNT_AUTH_QUERY: &str = r#"
    SELECT
        a.bank_acc AS number,
        a.bank_id,
        b.name AS bank_name,
        COALESCE(u.login, o.name, '') AS holder,
        a.bank_pass AS pin,
        a.cash
    FROM bank_accounts a
    JOIN organizations b ON b.id = a.bank_id
    LEFT JOIN users u ON u.id = a.bank_user
    LEFT JOIN organizations o ON o.id = a.owner_organization_id
    WHERE a.bank_acc = $1
"#;

pub async fn find_account_by_number(
    pool: &PgPool,
    account: i64,
) -> Result<Option<AccountAuthRow>, sqlx::Error> {
    sqlx::query_as::<_, AccountAuthRow>(ACCOUNT_AUTH_QUERY)
        .bind(account)
        .fetch_optional(pool)
        .await
}

pub async fn lock_account_by_number(
    tx: &mut Transaction<'_, Postgres>,
    account: i64,
) -> Result<Option<AccountAuthRow>, sqlx::Error> {
    let query = format!("{ACCOUNT_AUTH_QUERY} FOR UPDATE OF a");

    sqlx::query_as::<_, AccountAuthRow>(&query)
        .bind(account)
        .fetch_optional(&mut **tx)
        .await
}

pub async fn set_cash_by_number(
    tx: &mut Transaction<'_, Postgres>,
    account: i64,
    cash: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE bank_accounts SET cash = $2 WHERE bank_acc = $1")
        .bind(account)
        .bind(cash)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

pub async fn insert_transaction(
    tx: &mut Transaction<'_, Postgres>,
    account: i64,
    other: Option<i64>,
    amount: i64,
    balance: i64,
    concept: &str,
    origin_ip: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO bank_transactions(account, other_account, amount, balance, concept, origin_ip)
        VALUES($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(account)
    .bind(other)
    .bind(amount)
    .bind(balance)
    .bind(concept)
    .bind(origin_ip)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn find_transactions(
    pool: &PgPool,
    account: i64,
) -> Result<Vec<TransactionRow>, sqlx::Error> {
    sqlx::query_as::<_, TransactionRow>(
        r#"
        SELECT account, other_account, amount, balance, concept, origin_ip, created_at
        FROM bank_transactions
        WHERE account = $1
        ORDER BY created_at DESC, id DESC
        LIMIT 100
        "#,
    )
    .bind(account)
    .fetch_all(pool)
    .await
}

const CHARGE_COLUMNS: &str = r#"
    SELECT id, to_account, from_account, amount, concept, status, paid_by, created_at, paid_at
    FROM bank_charges
"#;

pub async fn insert_charge(
    pool: &PgPool,
    to_account: i64,
    from_account: Option<i64>,
    amount: i64,
    concept: &str,
) -> Result<ChargeRow, sqlx::Error> {
    sqlx::query_as::<_, ChargeRow>(
        r#"
        INSERT INTO bank_charges(to_account, from_account, amount, concept)
        VALUES($1, $2, $3, $4)
        RETURNING id, to_account, from_account, amount, concept, status, paid_by, created_at, paid_at
        "#,
    )
    .bind(to_account)
    .bind(from_account)
    .bind(amount)
    .bind(concept)
    .fetch_one(pool)
    .await
}

pub async fn find_charge(
    pool: &PgPool,
    id: i64,
) -> Result<Option<ChargeRow>, sqlx::Error> {
    let query = format!("{CHARGE_COLUMNS} WHERE id = $1");

    sqlx::query_as::<_, ChargeRow>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_charges(
    pool: &PgPool,
    account: i64,
) -> Result<Vec<ChargeRow>, sqlx::Error> {
    let query = format!(
        "{CHARGE_COLUMNS} WHERE to_account = $1 OR from_account = $1 \
         ORDER BY created_at DESC LIMIT 100"
    );

    sqlx::query_as::<_, ChargeRow>(&query)
        .bind(account)
        .fetch_all(pool)
        .await
}

/// Cerrar el cobro y quedarse con el sitio. El `status = 'pending'` del WHERE es lo que hace
/// que dos pagos a la vez no se cobren dos veces: solo uno escribe.
pub async fn mark_charge_paid(
    pool: &PgPool,
    id: i64,
    payer: i64,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        UPDATE bank_charges
        SET status = 'paid', paid_by = $2, paid_at = now()
        WHERE id = $1 AND status = 'pending'
        "#,
    )
    .bind(id)
    .bind(payer)
    .execute(pool)
    .await?;

    Ok(done.rows_affected())
}

/// Deshacer el cierre cuando el dinero no llego a moverse.
pub async fn reopen_charge(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE bank_charges
        SET status = 'pending', paid_by = NULL, paid_at = NULL
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn cancel_charge(
    pool: &PgPool,
    id: i64,
    issuer: i64,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        UPDATE bank_charges
        SET status = 'cancelled'
        WHERE id = $1 AND to_account = $2 AND status = 'pending'
        "#,
    )
    .bind(id)
    .bind(issuer)
    .execute(pool)
    .await?;

    Ok(done.rows_affected())
}

/// Desde que maquina se ordena. Se consulta solo en las operaciones que dejan rastro, para
/// no cobrarle una consulta de mas a todas las demas.
pub async fn player_ip(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT game_ip FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}
