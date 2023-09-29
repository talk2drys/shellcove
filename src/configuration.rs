use super::error::SCError;
use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Setting {
    pub listen_addr: String,
    pub listen_port: u16,
}

pub fn get_configuration() -> Result<Setting, SCError> {
    let config = Config::builder()
        .set_default("listen_addr", "127.0.0.1")?
        .set_default("listen_port", "8080")?
        .add_source(File::new("conf/default", FileFormat::Toml).required(true))
        .build()?;

    config.try_deserialize::<Setting>().map_err(SCError::Config)
}
