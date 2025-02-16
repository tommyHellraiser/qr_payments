use actix_web::{get, web, HttpResponse};
use the_logger::{log_error, log_info, TheLogger};

use crate::{datatypes::WalletsIdType, get_conn_or_internal_error, modules::wallets::Wallet};

pub fn wallets_services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_wallets).service(get_wallet);
}

/// /v1/wallets
#[get("")]
async fn get_wallets() -> HttpResponse {
    let logger = TheLogger::instance();

    log_info!(logger, "Selecting wallets...");

    let mut conn = get_conn_or_internal_error!(logger);

    let wallets = match Wallet::select_all(&mut conn) {
        Ok(wallets) => wallets,
        Err(error) => {
            log_error!(logger, "Could not get wallets: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    log_info!(logger, "Wallets selected successfully!");

    HttpResponse::Ok().json(wallets)
}

/// /v1/wallets
#[get("/{wallets_id}")]
async fn get_wallet(path: web::Path<WalletsIdType>) -> HttpResponse {
    let logger = TheLogger::instance();
    let wallets_id = path.into_inner();

    log_info!(logger, "Selecting wallet with ID: {}", wallets_id);

    let mut conn = get_conn_or_internal_error!(logger);

    let wallet = match Wallet::select_by_id(&mut conn, wallets_id) {
        Ok(wallet) => wallet,
        Err(error) => {
            log_error!(logger, "Could not get wallet: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Some(wallet) = wallet {
        log_info!(logger, "Wallet found. Sending Ok response");
        return HttpResponse::Ok().json(wallet);
    }

    let msg = format!("Wallet with ID: {} was not found", wallets_id);
    log_info!(logger, "{}", msg);
    HttpResponse::NotFound().json(msg)
}
