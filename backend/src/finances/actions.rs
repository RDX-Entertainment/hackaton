//! Lo que se puede hacer en un banco, sin importar quien lo pida ni desde donde.
//!
//! Este modulo es la capa que el navegador del juego va a consumir: la web de un banco, el
//! panel de un NPC o la pantalla de Dinero llaman todas aqui. Por eso ninguna funcion sabe
//! nada de sesiones ni de «el jugador conectado».
//!
//! **La credencial es la cuenta y su PIN, no la propiedad.** Es la diferencia que hace que
//! el juego tenga sentido: si moviera el dinero solo el titular, robar unas credenciales no
//! serviria de nada. Aqui, quien las trae manda, y el rastro que queda en
//! `bank_transactions` es lo que le permite a la victima descubrir lo que paso.
//!
//! Cada operacion deja escrito desde que IP se ordeno cuando quien llama la conoce.

use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        AccountAccessResponse,
        ChargeResponse,
        TransactionResponse,
    },
    repo,
};

/// El maximo que admite un concepto, igual que la columna.
const CONCEPT_MAX: usize = 120;

/// Comprobar unas credenciales es «entrar» en la cuenta: devuelve lo que se ve nada mas
/// entrar —de quien es y cuanto hay— y nada mas.
///
/// Es la operacion sobre la que se apoyan las demas, y tambien la que usa la web del banco
/// para dejar pasar a alguien.
pub async fn authenticate(
    pool: &PgPool,
    account: i64,
    pin: &str,
) -> Result<AccountAccessResponse, ApiError> {
    // Los tres motivos por los que esto puede fallar —la cuenta no existe, todavia no tiene
    // PIN, o el PIN no cuadra— responden **lo mismo**. Distinguirlos le diria a quien prueba
    // numeros cuales existen, y enumerar cuentas es el primer paso para vaciarlas.
    let found = repo::find_account_by_number(pool, account)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    if found.pin.as_deref() != Some(pin) {
        return Err(ApiError::Unauthorized);
    }

    Ok(AccountAccessResponse {
        account: found.number,
        bank_id: found.bank_id,
        bank_name: found.bank_name,
        holder: found.holder,
        cash: found.cash,
    })
}

/// Mover dinero de una cuenta a otra.
///
/// Quien trae el PIN de origen manda: no se comprueba de quien es la cuenta. El importe se
/// decide sobre el saldo bloqueado, no sobre el que se leyo antes.
pub async fn transfer(
    pool: &PgPool,
    from: i64,
    pin: &str,
    to: i64,
    amount: i64,
    concept: &str,
    origin_ip: Option<i64>,
) -> Result<TransactionResponse, ApiError> {
    if amount <= 0 {
        return Err(ApiError::BadRequest(
            "el importe tiene que ser mayor que cero".into(),
        ));
    }

    if from == to {
        return Err(ApiError::BadRequest(
            "el origen y el destino son la misma cuenta".into(),
        ));
    }

    let concept = clean_concept(concept)?;

    let mut tx = pool.begin().await?;

    // Se bloquea siempre en el mismo orden, de menor a mayor numero de cuenta. Dos
    // transferencias cruzadas a la vez —A a B y B a A— se bloquearian en orden contrario y
    // PostgreSQL tendria que matar una por interbloqueo.
    let (first, second) =
        if from < to { (from, to) } else { (to, from) };

    let first_row = repo::lock_account_by_number(&mut tx, first)
        .await?
        .ok_or(ApiError::NotFound)?;
    let second_row = repo::lock_account_by_number(&mut tx, second)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (source, target) = if first_row.number == from {
        (first_row, second_row)
    } else {
        (second_row, first_row)
    };

    if source.pin.as_deref() != Some(pin) {
        return Err(ApiError::Unauthorized);
    }

    if source.cash < amount {
        return Err(ApiError::BadRequest(
            "insufficient funds: no hay tanto en esa cuenta".into(),
        ));
    }

    let source_left = source.cash - amount;
    let target_left = target.cash + amount;

    repo::set_cash_by_number(&mut tx, from, source_left).await?;
    repo::set_cash_by_number(&mut tx, to, target_left).await?;

    repo::insert_transaction(
        &mut tx, from, Some(to), -amount, source_left, &concept, origin_ip,
    )
    .await?;
    repo::insert_transaction(
        &mut tx, to, Some(from), amount, target_left, &concept, origin_ip,
    )
    .await?;

    tx.commit().await?;

    Ok(TransactionResponse {
        account: from,
        other_account: Some(to),
        amount: -amount,
        balance: source_left,
        concept,
    })
}

/// Crear un cobro contra otra cuenta, o abierto para quien lo tenga delante.
///
/// Solo hace falta el PIN de **la cuenta que cobra**: pedir dinero no mueve nada, asi que no
/// hay nada que autorizar del otro lado. Quien decide es el que paga.
pub async fn create_charge(
    pool: &PgPool,
    to_account: i64,
    pin: &str,
    from_account: Option<i64>,
    amount: i64,
    concept: &str,
) -> Result<ChargeResponse, ApiError> {
    if amount <= 0 {
        return Err(ApiError::BadRequest(
            "el importe tiene que ser mayor que cero".into(),
        ));
    }

    if from_account == Some(to_account) {
        return Err(ApiError::BadRequest(
            "no se puede cobrar a la misma cuenta que cobra".into(),
        ));
    }

    let concept = clean_concept(concept)?;

    authenticate(pool, to_account, pin).await?;

    if let Some(payer) = from_account {
        if repo::find_account_by_number(pool, payer).await?.is_none() {
            return Err(ApiError::NotFound);
        }
    }

    let charge = repo::insert_charge(
        pool, to_account, from_account, amount, &concept,
    )
    .await?;

    Ok(charge.into())
}

/// Pagar un cobro. El que paga trae su cuenta y su PIN, y con eso se hace la transferencia.
///
/// Un cobro dirigido solo lo paga su destinatario; uno abierto lo paga cualquiera. Se marca
/// como pagado **antes** de mover el dinero, y con la condicion de que siguiera pendiente:
/// si dos pagan a la vez, el segundo se encuentra el cobro cerrado y no paga dos veces.
pub async fn pay_charge(
    pool: &PgPool,
    charge_id: i64,
    payer: i64,
    pin: &str,
    origin_ip: Option<i64>,
) -> Result<TransactionResponse, ApiError> {
    let charge = repo::find_charge(pool, charge_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if charge.status != "pending" {
        return Err(ApiError::BadRequest(
            "ese cobro ya no esta pendiente".into(),
        ));
    }

    if let Some(target) = charge.from_account {
        if target != payer {
            return Err(ApiError::BadRequest(
                "ese cobro va dirigido a otra cuenta".into(),
            ));
        }
    }

    if charge.to_account == payer {
        return Err(ApiError::BadRequest(
            "no puedes pagarte un cobro a ti mismo".into(),
        ));
    }

    // Cerrar primero y mover despues. Si la transferencia falla, el cobro se reabre.
    if repo::mark_charge_paid(pool, charge_id, payer).await? == 0 {
        return Err(ApiError::BadRequest(
            "ese cobro ya no esta pendiente".into(),
        ));
    }

    let moved = transfer(
        pool,
        payer,
        pin,
        charge.to_account,
        charge.amount,
        &charge.concept,
        origin_ip,
    )
    .await;

    match moved {
        Ok(receipt) => Ok(receipt),
        Err(error) => {
            repo::reopen_charge(pool, charge_id).await?;
            Err(error)
        }
    }
}

/// Retirar un cobro que todavia nadie ha pagado. Solo lo hace quien lo emitio.
pub async fn cancel_charge(
    pool: &PgPool,
    charge_id: i64,
    account: i64,
    pin: &str,
) -> Result<(), ApiError> {
    authenticate(pool, account, pin).await?;

    if repo::cancel_charge(pool, charge_id, account).await? == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

/// Los cobros que tocan a esta cuenta: los que emitio y los que le reclaman.
pub async fn charges(
    pool: &PgPool,
    account: i64,
    pin: &str,
) -> Result<Vec<ChargeResponse>, ApiError> {
    authenticate(pool, account, pin).await?;

    Ok(repo::find_charges(pool, account)
        .await?
        .into_iter()
        .map(ChargeResponse::from)
        .collect())
}

/// El extracto: lo que ha entrado y salido, y desde donde se ordeno.
pub async fn statement(
    pool: &PgPool,
    account: i64,
    pin: &str,
) -> Result<Vec<TransactionResponse>, ApiError> {
    authenticate(pool, account, pin).await?;

    Ok(repo::find_transactions(pool, account)
        .await?
        .into_iter()
        .map(TransactionResponse::from)
        .collect())
}

/// Un concepto es texto y nada mas. Se recorta y se rechaza si se pasa de largo, en vez de
/// dejar que lo trunque la base sin avisar.
fn clean_concept(concept: &str) -> Result<String, ApiError> {
    let clean = concept.trim();

    if clean.chars().count() > CONCEPT_MAX {
        return Err(ApiError::BadRequest(format!(
            "el concepto no puede pasar de {CONCEPT_MAX} caracteres"
        )));
    }

    Ok(clean.to_owned())
}
