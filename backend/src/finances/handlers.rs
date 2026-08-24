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
    dtos::{
        BankTransferRequest,
        BitcoinTransferRequest,
        CreateBankAccountRequest,
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
        .route(
            "/finances/accounts",
            web::get().to(bank_accounts),
        )
        .route(
            "/finances/accounts",
            web::post().to(create_bank_account),
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
