use std::{fs::File, sync::OnceLock};
use error_mapper::{create_new_error, TheResult};
use serde::Deserialize;
use tokio::sync::RwLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Default)]
pub struct Config {
    inner: RwLock<ConfigInner>
}

#[derive(Deserialize, Debug, Clone, Default)]
struct ConfigInner {
    db: DbConfig
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct DbConfig {
    user: String,
    pass: String,
    addr: String,
    db_name: String
}

impl Config {
    pub fn initialize_config() {
        let file = File::open("config/config.json").unwrap();
        let config = serde_json::from_reader::<_, ConfigInner>(file).unwrap();
        let config = Config { inner: RwLock::new(config) };

        CONFIG.get_or_init(|| config);
    }

    pub async fn get_db_config() -> TheResult<DbConfig> {
        let config = CONFIG.get().ok_or_else(|| create_new_error!("Could not find Db Configurations"))?;
        Ok(config.inner.read().await.db.clone())
    }
}

impl DbConfig {
    pub fn build_conn_string(&self) -> String {
        format!("mysql://{}:{}@{}/{}", self.user, self.pass, self.addr, self.db_name)
    }
}
