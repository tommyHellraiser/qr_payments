use config::Config;
use database::DbConn;
use the_logger::{log_error, TheLogger};

mod config;
mod database;
mod datatypes;
mod modules;

#[tokio::main]
async fn main() {
    Config::initialize_config();
    let logger = TheLogger::instance();
    if let Err(error) = DbConn::init_connection().await {
        log_error!(logger, "Could not initialize Db Connection: {}", error);
        return
    }

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
    }
}
