use actix_web::{post, web, HttpResponse};
use rust_decimal::Decimal;
use serde::Deserialize;
use the_logger::{log_critical, log_error, log_info, TheLogger};

use crate::{
    datatypes::WalletsIdType,
    get_conn_or_internal_error,
    modules::{
        transactions::{Transaction, TransactionStatus},
        wallets::Wallet,
    },
};

pub fn transactions_services(cfg: &mut web::ServiceConfig) {
    cfg.service(new_transaction).service(confirm_transaction);
}

#[derive(Deserialize)]
struct NewTransactionRequest {
    wallets_id: WalletsIdType,
    amount: Decimal,
}

#[derive(Deserialize)]
struct PostTransactionRequest {
    wallets_id: WalletsIdType,
    transaction_token: String,
}

#[post("")]
async fn new_transaction(body: web::Json<NewTransactionRequest>) -> HttpResponse {
    let logger = TheLogger::instance();
    let mut conn = get_conn_or_internal_error!(logger);
    let body = body.into_inner();

    log_info!(logger, "Received new transaction request. Processing");

    let mut transaction = Transaction::new(body.amount);

    //  Validate wallet exists
    let mut wallet = match Wallet::select_by_id(&mut conn, body.wallets_id) {
        Ok(Some(wallet)) => wallet,
        Ok(None) => {
            let msg = format!("Invalid wallet received with ID: {}", body.wallets_id);
            log_info!(logger, "{}", msg);
            transaction.errors = Some(msg);

            match transaction.log(&mut conn) {
                Ok(true) => {}
                Ok(false) => {
                    log_info!(logger, "Could not log transaction. No details available");
                }
                Err(error) => {
                    log_error!(logger, "Could not log transaction: {}", error);
                }
            }

            return HttpResponse::BadRequest().json(format!(
                "Wallet with ID: {} does not exist. Aborting transaction",
                body.wallets_id
            ));
        }
        Err(error) => {
            log_error!(logger, "Error selecting wallet: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    //  Validate transaction amount first
    if let Some(error) = transaction.validate() {
        log_info!(logger, "Invalid transaction: {}", error);
        return HttpResponse::PreconditionFailed().json(error);
    }

    //  Then validate wallet balance
    if !wallet.validate_balance(body.amount) {
        let msg = "Wallet has insufficient balance for transaction";
        log_info!(logger, "{}", msg);
        return HttpResponse::PreconditionFailed().json(msg);
    }

    //  If everything is okay, generate token and proceed with transaction
    transaction.generate_token();
    transaction.wallets_id = Some(wallet.id);
    let token = match transaction.insert(&mut conn) {
        Ok(id) => id,
        Err(error) => {
            log_error!(logger, "Could not insert transaction: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    //  Affect wallet balance
    match wallet.affect_balance(&mut conn, body.amount) {
        Ok(true) => {
            log_info!(
                logger,
                "Transaction approved, continue to confirmation stage"
            );
            return HttpResponse::Ok().json(token);
        }
        Ok(false) => {
            log_error!(
                logger,
                "Could not affect wallet balance, rolling back transaction"
            );
            transaction.errors = Some(String::from("Could not affect wallet balance"));
        }
        Err(error) => {
            log_error!(
                logger,
                "Error affecting wallet balance, rolling back transaction: {}",
                error
            );
            transaction.errors = Some(String::from("MySql error while affecting wallet balance"));
        }
    }

    //  From this point onwards, the processing is to handle an error state. Every response will be 500
    transaction.status = TransactionStatus::InternalError;

    //  If roll back fails, log a crit error
    let crit_error = match transaction.update_status_and_error(&mut conn) {
        Ok(true) => None,
        Ok(false) => Some(String::from("Unable to roll back transaction")),
        Err(error) => Some(format!("Error while rolling back transaction: {}", error)),
    };

    if let Some(error) = crit_error {
        log_critical!(
            logger,
            "Critical error while rolling back transaction: {}",
            error
        )
    } else {
        log_info!(logger, "Transaction rolled back successfully");
    }

    HttpResponse::InternalServerError().finish()
}

#[post("/confirm")]
async fn confirm_transaction(body: web::Json<PostTransactionRequest>) -> HttpResponse {
    let body = body.into_inner();
    let logger = TheLogger::instance();
    let mut conn = get_conn_or_internal_error!(logger);

    log_info!(logger, "Received Confirm Transaction request");

    let mut transaction = match Transaction::select_by_token_and_wallets_id(
        &mut conn,
        body.wallets_id,
        body.transaction_token,
    ) {
        Ok(Some(transaction)) => transaction,
        Ok(None) => {
            log_info!(logger, "No transaction found for received params");
            return HttpResponse::NotFound().json("No transaction found for the required criteria");
        }
        Err(error) => {
            log_error!(logger, "Error searching for transaction: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    //  Transaction found, confirming it
    transaction.status = TransactionStatus::Confirmed;
    match transaction.update_status_and_error(&mut conn) {
        Ok(true) => {}
        Ok(false) => {
            log_error!(
                logger,
                "Could not confirm transaction. No extra details provided"
            );
            return HttpResponse::InternalServerError().finish();
        }
        Err(error) => {
            log_error!(logger, "Could not confirm transaction: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }

    log_info!(logger, "Transaction confirmed");

    HttpResponse::Ok().finish()
}
