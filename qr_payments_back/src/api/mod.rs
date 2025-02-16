use actix_web::{dev::ServerHandle, web, App, HttpServer};
use error_mapper::{create_new_error, TheResult};
use the_logger::{log_info, TheLogger};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{config::Config, TIME_FORMAT};

mod services;

pub struct ApiData {
    stop_signal: mpsc::Sender<()>,
}

pub async fn start_api(stop_channels: (Sender<()>, Receiver<()>)) -> TheResult<()> {
    let api_config = Config::get_api_config().await?;

    let server =
        HttpServer::new(move || {
            let stop_sender = stop_channels.0.clone();
            App::new()
                .app_data(web::Data::new(ApiData {
                    stop_signal: stop_sender,
                }))
                .service(
                    web::scope("/v1")
                        .service(web::scope("/configurations").configure(services::api_services))
                        .service(
                            web::scope("/wallets")
                                .configure(crate::modules::wallets::services::wallets_services),
                        )
                        .service(web::scope("/transactions").configure(
                            crate::modules::transactions::services::transactions_services,
                        )),
                )
        })
        .bind((api_config.addr.as_str(), api_config.port))
        .map_err(|error| create_new_error!(format!("Failed to initialize HTTP server: {}", error)))?
        .workers(api_config.workers)
        .run();

    let server_handler = server.handle();
    tokio::spawn(api_stop_handler(server_handler, stop_channels.1));

    server
        .await
        .map_err(|error| create_new_error!(error.to_string()))
}

async fn api_stop_handler(server_handler: ServerHandle, mut stop_receiver: Receiver<()>) {
    stop_receiver.recv().await;

    let logger = TheLogger::instance();
    log_info!(
        logger,
        "Initializing shutdown at: {}",
        chrono::Local::now().naive_local().format(TIME_FORMAT)
    );

    server_handler.stop(true).await;

    log_info!(
        logger,
        "Successfully killed server at: {}",
        chrono::Local::now().naive_local().format(TIME_FORMAT)
    );
}
