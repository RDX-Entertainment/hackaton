use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    auth::Ctx,
    error::ApiError,
    state::AppState,
};

use super::{
    actions,
    dtos::{
        AdjustCashRequest,
        AuthenticateRequest,
        BankTransferOrder,
        BankTransferRequest,
        CreateChargeRequest,
        PayChargeRequest,
        BitcoinTransferRequest,
        CreateBankAccountRequest,
        SetPinRequest,
    },
    services,
};

//
// ==============================
// ROUTES
// ==============================
//

pub fn configure(
    cfg: &mut web::ServiceConfig,
) {
    cfg
        .route("/finances/accounts/{id}/main", web::post().to(set_main))
        .route("/finances/main", web::get().to(main_account))
        .route("/finances/main/statement", web::get().to(main_statement))
        .route("/finances/main/transfer", web::post().to(main_transfer))
        .route(
            "/finances/banks",
            web::get().to(banks),
        )
        .route(
            "/finances/accounts",
            web::get().to(bank_accounts),
        )
        .route(
            "/finances/accounts",
            web::post().to(create_bank_account),
        )
        .route(
            "/finances/accounts/{id}/pin",
            web::post().to(set_pin),
        )
        .route(
            "/finances/transfers",
            web::post().to(bank_transfer),
        )
        .route(
            "/bitcoin/wallets",
            web::get().to(bitcoin_wallets),
        )
        .route(
            "/bitcoin/wallets",
            web::post().to(create_bitcoin_wallet),
        )
        .route(
            "/bitcoin/transfers",
            web::post().to(bitcoin_transfer),
        );
}

/// Las operaciones del banco, tal como las va a pedir el navegador del juego.
///
/// Aqui **la credencial es la cuenta y su PIN**, no la sesion: hace falta estar dentro del
/// juego para llamar, pero lo que se mueve no es «tu» dinero sino el de las credenciales que
/// traigas. Es lo que permite que unas credenciales robadas sirvan de algo.
///
/// Se monta bajo `/api/bank`.
pub fn configure_bank(
    cfg: &mut web::ServiceConfig,
) {
    cfg
        .route(
            "/bank/login",
            web::post().to(bank_login),
        )
        .route(
            "/bank/statement",
            web::post().to(bank_statement),
        )
        .route(
            "/bank/transfer",
            web::post().to(bank_transfer_order),
        )
        .route(
            "/bank/charges",
            web::post().to(bank_charges),
        )
        .route(
            "/bank/charges/new",
            web::post().to(bank_create_charge),
        )
        .route(
            "/bank/charges/{id}/pay",
            web::post().to(bank_pay_charge),
        )
        .route(
            "/bank/charges/{id}/cancel",
            web::post().to(bank_cancel_charge),
        );
}

/// La IP desde la que se ordena, para que el movimiento deje rastro investigable. Que no se
/// sepa no puede tumbar la operacion: el rastro es deseable, no obligatorio.
async fn ordered_from(state: &AppState, ctx: &Ctx) -> Option<i64> {
    super::repo::player_ip(&state.pool, ctx.id)
        .await
        .ok()
        .flatten()
}

async fn set_main(state: web::Data<AppState>, ctx: Ctx, id: web::Path<i64>) -> Result<HttpResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE").bind(ctx.id).execute(&mut *tx).await?;
    let changed = sqlx::query("UPDATE users SET main_bank_account=$2 WHERE id=$1 AND EXISTS(SELECT 1 FROM bank_accounts WHERE id=$2 AND bank_user=$1)")
        .bind(ctx.id).bind(id.into_inner()).execute(&mut *tx).await?.rows_affected();
    if changed == 0 { return Err(ApiError::NotFound); }
    tx.commit().await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn main_account(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(actions::main_access(&state.pool, ctx.id).await?))
}

async fn main_statement(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    let access = actions::main_access(&state.pool, ctx.id).await?;
    let rows: Vec<super::dtos::TransactionResponse> = super::repo::find_transactions(&state.pool, access.account).await?
        .into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(rows))
}

#[derive(serde::Deserialize)]
struct MainTransfer { from_account: i64, to_account: i64, amount: i64, #[serde(default)] concept: String }

async fn main_transfer(state: web::Data<AppState>, ctx: Ctx, input: web::Json<MainTransfer>) -> Result<HttpResponse, ApiError> {
    crate::onboarding::services::guard(&state.pool, ctx.id, "banco.transferir").await?;
    let ip = ordered_from(&state, &ctx).await;
    Ok(HttpResponse::Ok().json(actions::transfer_main(&state.pool, ctx.id, input.from_account, input.to_account, input.amount, &input.concept, ip).await?))
}

pub async fn bank_login(
    state: web::Data<AppState>,
    _ctx: Ctx,
    input: web::Json<AuthenticateRequest>,
) -> Result<HttpResponse, ApiError> {
    let input = input.into_inner();
    let response = actions::authenticate(
        &state.pool,
        input.account,
        &input.pin,
    )
    .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn bank_statement(
    state: web::Data<AppState>,
    _ctx: Ctx,
    input: web::Json<AuthenticateRequest>,
) -> Result<HttpResponse, ApiError> {
    let input = input.into_inner();
    let response = actions::statement(
        &state.pool,
        input.account,
        &input.pin,
    )
    .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn bank_transfer_order(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<BankTransferOrder>,
) -> Result<HttpResponse, ApiError> {
    let from = ordered_from(&state, &ctx).await;
    let input = input.into_inner();

    let response = actions::transfer(
        &state.pool,
        input.account,
        &input.pin,
        input.to_account,
        input.amount,
        &input.concept,
        from,
    )
    .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn bank_charges(
    state: web::Data<AppState>,
    _ctx: Ctx,
    input: web::Json<AuthenticateRequest>,
) -> Result<HttpResponse, ApiError> {
    let input = input.into_inner();
    let response = actions::charges(
        &state.pool,
        input.account,
        &input.pin,
    )
    .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn bank_create_charge(
    state: web::Data<AppState>,
    _ctx: Ctx,
    input: web::Json<CreateChargeRequest>,
) -> Result<HttpResponse, ApiError> {
    let input = input.into_inner();
    let response = actions::create_charge(
        &state.pool,
        input.account,
        &input.pin,
        input.from_account,
        input.amount,
        &input.concept,
    )
    .await?;

    Ok(HttpResponse::Created().json(response))
}

pub async fn bank_pay_charge(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<i64>,
    input: web::Json<PayChargeRequest>,
) -> Result<HttpResponse, ApiError> {
    let from = ordered_from(&state, &ctx).await;
    let input = input.into_inner();

    let response = actions::pay_charge(
        &state.pool,
        path.into_inner(),
        input.account,
        &input.pin,
        from,
    )
    .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn bank_cancel_charge(
    state: web::Data<AppState>,
    _ctx: Ctx,
    path: web::Path<i64>,
    input: web::Json<AuthenticateRequest>,
) -> Result<HttpResponse, ApiError> {
    let input = input.into_inner();

    actions::cancel_charge(
        &state.pool,
        path.into_inner(),
        input.account,
        &input.pin,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Se monta dentro de `/api/admin`, detras de `AdminOnly`. Las cuentas de una organizacion
/// las abre el panel: no hay jugador que las pida.
pub fn configure_admin(
    cfg: &mut web::ServiceConfig,
) {
    cfg
        .route(
            "/organizations/{id}/accounts",
            web::get().to(organization_accounts),
        )
        .route(
            "/organizations/{id}/accounts",
            web::post().to(open_organization_account),
        )
        .route(
            "/users/{id}/accounts",
            web::get().to(user_accounts),
        )
        .route(
            "/users/{id}/accounts",
            web::post().to(open_user_account),
        )
        .route(
            "/accounts/{id}/cash",
            web::post().to(adjust_cash),
        );
}

pub async fn user_accounts(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    ctx.require_admin()?;

    let response = services::user_accounts(
        &state.pool,
        path.into_inner(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn open_user_account(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<i64>,
    input: web::Json<CreateBankAccountRequest>,
) -> Result<HttpResponse, ApiError> {
    ctx.require_admin()?;

    let response = services::open_user_account(
        &state.pool,
        path.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(HttpResponse::Created().json(response))
}

pub async fn adjust_cash(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<i64>,
    input: web::Json<AdjustCashRequest>,
) -> Result<HttpResponse, ApiError> {
    ctx.require_admin()?;

    let response = services::adjust_cash(
        &state.pool,
        path.into_inner(),
        input.into_inner().amount,
    )
    .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn set_pin(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<i64>,
    input: web::Json<SetPinRequest>,
) -> Result<HttpResponse, ApiError> {
    services::set_pin(
        &state.pool,
        ctx.id,
        path.into_inner(),
        input.into_inner().password,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn banks(
    state: web::Data<AppState>,
    _ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::banks(&state.pool).await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn organization_accounts(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    ctx.require_admin()?;

    let response =
        services::organization_accounts(
            &state.pool,
            path.into_inner(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn open_organization_account(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<i64>,
    input: web::Json<CreateBankAccountRequest>,
) -> Result<HttpResponse, ApiError> {
    ctx.require_admin()?;

    let response =
        services::open_organization_account(
            &state.pool,
            path.into_inner(),
            input.into_inner(),
        )
        .await?;

    Ok(HttpResponse::Created().json(response))
}

//
// ==============================
// BANK ACCOUNTS
// ==============================
//

pub async fn bank_accounts(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::accounts(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn create_bank_account(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<CreateBankAccountRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::open_account(
        &state.pool,
        ctx.id,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

//
// ==============================
// BANK TRANSFERS
// ==============================
//

pub async fn bank_transfer(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<BankTransferRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::transfer(
        &state.pool,
        ctx.id,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// BITCOIN
// ==============================
//

pub async fn bitcoin_wallets(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::wallets(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn create_bitcoin_wallet(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::create_wallet(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

pub async fn bitcoin_transfer(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<BitcoinTransferRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::transfer_bitcoin(
        &state.pool,
        ctx.id,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}
