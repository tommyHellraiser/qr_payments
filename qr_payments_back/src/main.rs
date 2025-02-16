use chrono::NaiveDateTime;
use config::Config;
use database::DbConn;
use error_mapper::{create_new_error, TheResult};
use std::sync::OnceLock;
use the_logger::{log_error, TheLogger};
use tokio::sync::mpsc::{Receiver, Sender};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
static ALIVE_SINCE: OnceLock<NaiveDateTime> = OnceLock::new();
const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const TIME_FORMAT: &str = "%H:%M:%S";

mod api;
mod config;
mod database;
mod datatypes;
mod modules;

#[tokio::main]
async fn main() {
    let stop_channels = match init_configurations().await {
        Ok(channels) => channels,
        Err(error) => {
            let logger = TheLogger::instance();
            log_error!(logger, "{}", error);
            return;
        }
    };

    if let Err(error) = api::start_api(stop_channels).await {
        let logger = TheLogger::instance();
        log_error!(logger, "Error starting Api: {}", error);
        return;
    };
}

async fn init_configurations() -> TheResult<(Sender<()>, Receiver<()>)> {
    ALIVE_SINCE.get_or_init(|| chrono::Local::now().naive_local());
    Config::initialize_config();
    if let Err(error) = DbConn::init_connection().await {
        let error = format!("Could not initialize Db Connection: {}", error);
        return Err(create_new_error!(error));
    }

    let (stop_sender, stop_receiver) = tokio::sync::mpsc::channel::<()>(5);

    Ok((stop_sender, stop_receiver))
}

/// ## Description
/// Macro that extracts any primitive, plus Strings values from a Row element
///
/// ### Parameters
/// - row: row element in FromRow implementation
/// - field: column name in database table
/// - table: table name in database
/// - datatype: the type of the data to be converted from the Row element
#[macro_export]
macro_rules! row_to_data {
    ($row:ident, $field:expr, $table:expr, $datatype:ty) => {
        match $row.get::<$datatype, _>($field) {
            Some(value) => value,
            None => {
                panic!("Unknown column {} in table {}", $field.to_string(), $table);
            }
        }
    };
}
