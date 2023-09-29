use crate::error::SCError;
use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "Result<SSHMessageResponse, SCError>")]
#[serde(tag = "type", content = "payload")]
pub enum SSHMessage {
    Connect {
        host: String,
        port: u16,
        username: String,
        password: String,
        jump_host: Option<SSHCredential>,
    },
    ShellInput(Vec<u8>),
    TerminalResize {
        width: i32,
        height: i32,
        pixelwidth: i32,
        pixelheight: i32,
    },
    ShellOutput(Vec<u8>),
    // Disconnect,
}

#[derive(Debug, Clone)]
pub enum SSHMessageResponse {
    Connected,
    PermissionDenied,
    SSHError,
    NoOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHCredential {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}
