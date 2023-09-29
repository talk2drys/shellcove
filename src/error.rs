#[derive(Debug)]
pub enum SCError {
    SSHError(russh::Error),
    PermissionDenied,
    ByteParseError,
    Config(config::ConfigError),
    Error,
}

impl From<config::ConfigError> for SCError {
    fn from(err: config::ConfigError) -> Self {
        SCError::Config(err)
    }
}
