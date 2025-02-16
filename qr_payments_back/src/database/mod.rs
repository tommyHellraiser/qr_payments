use std::sync::OnceLock;

use error_mapper::{create_new_error, TheResult};
use mysql::{Pool, PooledConn};

use crate::config::Config;

static MYSQL: OnceLock<DbConn> = OnceLock::new();

pub struct DbConn {
    pool: Pool
}

impl DbConn {
    pub async fn init_connection() -> TheResult<()> {
        let db_config = Config::get_db_config().await?;
        let conn_string = db_config.build_conn_string();

        let pool = Pool::new(conn_string.as_str()).map_err(|error| create_new_error!(error.to_string()))?;
        MYSQL.get_or_init(|| DbConn { pool });

        Ok(())
    }
    pub async fn get_conn() -> TheResult<PooledConn> {
        let db_conn = MYSQL.get().ok_or_else(|| create_new_error!("Failed to get Db Connection from Db Pool"))?;
        db_conn.pool.get_conn().map_err(|error| create_new_error!(error.to_string()))
    }
}
