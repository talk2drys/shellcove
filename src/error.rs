#[derive(Debug)]
pub enum SCError {
    Config(config::ConfigError),
}

impl From<config::ConfigError> for SCError {
    fn from(err: config::ConfigError) -> Self {
        SCError::Config(err)
    }
}
