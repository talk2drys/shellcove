use config::{Config, ConfigError};

#[derive(Debug, Clone)]
pub struct Setting {
    pub listen_addr: String,
    pub listen_port: u16,
}

pub fn get_configuration() -> Result<Setting, ConfigError> {
    let config = Config::builder()
        .set_default("listen_addr", "127.0.0.1")?
        .set_default("listen_port", "8080")?
        .build()?;

    Ok(config.into())
}
