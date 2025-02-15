use std::panic::panic_any;

use config::Config;

mod config;

#[tokio::main]
async fn main() {
    Config::initialize_config();

    let db_config = match Config::get_db_config().await {
        Ok(config) => config,
        Err(error) => panic_any(error.to_string())
    };
    
    let conn_string = db_config.build_conn_string();
    dbg!(&conn_string);
}
