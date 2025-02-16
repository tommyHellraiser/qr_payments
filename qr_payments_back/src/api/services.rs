use actix_web::{get, put, web, HttpResponse};
use the_logger::{log_error, log_info, TheLogger};

use crate::{api::ApiData, ALIVE_SINCE, APP_NAME, APP_VERSION, DATETIME_FORMAT, TIME_FORMAT};

pub(super) fn api_services(cfg: &mut web::ServiceConfig) {
    cfg.service(alive).service(stop);
}

/// v1/configurations/alive
#[get("/alive")]
pub(super) async fn alive() -> HttpResponse {
    let Some(alive_since) = ALIVE_SINCE
        .get()
        .map(|alive| alive.format(DATETIME_FORMAT).to_string())
    else {
        return HttpResponse::InternalServerError().finish();
    };
    let alive_msg = format!(
        "{}, version: {} | Alive since: {}",
        APP_NAME, APP_VERSION, alive_since
    );

    HttpResponse::Ok().body(alive_msg)
}

/// v1/configurations/stop
#[put("/stop")]
async fn stop(api_data: web::Data<ApiData>) -> HttpResponse {
    let logger = TheLogger::instance();

    if let Err(error) = api_data.stop_signal.send(()).await {
        log_error!(logger, "Failed to send stop signal: {}", error);
        return HttpResponse::InternalServerError().finish();
    };

    log_info!(
        logger,
        "Stop signal sent at: {}",
        chrono::Local::now().naive_local().format(TIME_FORMAT)
    );

    HttpResponse::Ok().finish()
}
