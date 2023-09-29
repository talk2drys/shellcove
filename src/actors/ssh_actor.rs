use super::ws_actor::WSActor;
use crate::error::SCError;
use crate::messages::{SSHMessage, SSHMessageResponse};
use actix::{Actor, ActorFutureExt, Addr, Context, ResponseActFuture, WrapFuture};
use async_trait::async_trait;
use russh::client::Msg;
use russh::{client, Channel, ChannelId, ChannelStream, Pty};
use russh_keys::*;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpSocket;
use tracing::{debug, info};

#[derive(Debug)]
pub struct SSHActor {
    pub channel: Option<Arc<Mutex<Channel<Msg>>>>,
    pub ws_addr: Option<actix::Addr<WSActor>>,
}

impl Actor for SSHActor {
    type Context = Context<Self>;
}

impl<'data> actix::Handler<SSHMessage> for SSHActor {
    type Result = ResponseActFuture<Self, Result<SSHMessageResponse, SCError>>;

    fn handle(&mut self, msg: SSHMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SSHMessage::Connect {
                host,
                port,
                username,
                password,
                jump_host,
            } => {
                debug!(
                    "SSH Message Connect: host={}, port={}, username={}",
                    host, port, username
                );
                let config = russh::client::Config::default();
                let config = Arc::new(config);
                let client = SSHClient {
                    actor_addr: self.ws_addr.clone().unwrap(),
                };

                let fut = async move {
                    let mut session = if let Some(proxy) = jump_host {
                        let proxy_client_handler = SSHProxyClient;
                        let mut session = russh::client::connect(
                            config.clone(),
                            (proxy.host, proxy.port),
                            proxy_client_handler,
                        )
                        .await
                        .unwrap();

                        // authenticate into the proxy server
                        let is_authenticated = session
                            .authenticate_password(proxy.username, proxy.password)
                            .await
                            .unwrap();

                        let ssh = session
                            .channel_open_direct_tcpip(host, port as u32, "", 0)
                            .await;

                        let stream = ssh.unwrap().into_stream();

                        russh::client::connect_stream(config, stream, client.clone())
                            .await
                            .unwrap()
                    } else {
                        let address: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
                        let socket = TcpSocket::new_v4().unwrap();
                        let socket = socket.connect(address).await.unwrap();

                        russh::client::connect_stream(config, socket, client)
                            .await
                            .unwrap()
                    };

                    // Now authenticate to the target ssh
                    match session.authenticate_password(username, password).await {
                        Err(err) => return Err(SCError::SSHError(err)),
                        Ok(isAuthnenticated) => {
                            if !isAuthnenticated {
                                println!("Permission Denied");
                                return Err(SCError::PermissionDenied);
                            }

                            let mut channel = match session.channel_open_session().await {
                                Ok(val) => val,
                                Err(err) => {
                                    println!("Error opening channel: {:?}", err);
                                    return Err(SCError::SSHError(err));
                                }
                            };

                            channel
                                .request_pty(
                                    true,
                                    "xterm",
                                    80,
                                    24,
                                    640,
                                    480,
                                    &[
                                        // (Pty::VERASE, 1),
                                        // (Pty::ECHOKE, 1),
                                        // (Pty::ONLCR, 1),
                                        // (Pty::ECHOE, 1),
                                    ],
                                )
                                .await
                                .unwrap();
                            channel.request_shell(true).await.unwrap();
                            channel.set_env(true, "TERM", "xterm").await.unwrap();

                            if let Some(msg) = channel.wait().await {
                                println!("{:?}", msg)
                            }
                            Ok(channel)
                        }
                    }
                };

                Box::pin(fut.into_actor(self).map(|res, act, _| {
                    if let Ok(channel) = res {
                        act.channel = Some(Arc::new(Mutex::new(channel)));
                        return Ok(SSHMessageResponse::Connected);
                    }

                    Err(SCError::Error)
                }))
            }
            SSHMessage::TerminalResize {
                width,
                height,
                pixelwidth,
                pixelheight,
            } => {
                if let Some(channel) = self.channel.as_ref() {
                    let cloned_channel = channel.clone();
                    let fut = async move {
                        let mut channel_guard = cloned_channel.lock().unwrap();
                        channel_guard
                            .window_change(
                                width as u32,
                                height as u32,
                                pixelwidth as u32,
                                pixelheight as u32,
                            )
                            .await
                            .unwrap();
                        Ok(SSHMessageResponse::NoOp)
                    };

                    Box::pin(fut.into_actor(self).map(|res, _, _| res))
                } else {
                    Box::pin(async move { Err(SCError::Error) }.into_actor(self).map(
                        |_res: Result<SSHMessageResponse, SCError>, _, _| {
                            Ok(SSHMessageResponse::NoOp)
                        },
                    ))
                }
            }
            SSHMessage::ShellInput(cmd) => {
                if let Some(channel) = self.channel.as_ref() {
                    let cloned_channel = channel.clone();
                    let fut = async move {
                        let mut channel_guard = cloned_channel.lock().unwrap();
                        if let Err(err) = channel_guard.data(cmd.as_slice()).await {
                            return Err(SCError::SSHError(err));
                        }

                        Ok(SSHMessageResponse::NoOp)
                    };

                    Box::pin(fut.into_actor(self).map(|res, _, _| res))
                } else {
                    Box::pin(async move { Err(SCError::Error) }.into_actor(self).map(
                        |res: Result<SSHMessageResponse, SCError>, _, _| {
                            Ok(SSHMessageResponse::NoOp)
                        },
                    ))
                }
            }
            SSHMessage::ShellOutput(data) => {
                if let Some(channel) = self.channel.as_ref() {
                    let cloned_channel = channel.clone();
                    let fut = async move {
                        let mut channel_guard = cloned_channel.lock().unwrap();
                        if let Err(err) = channel_guard.data(data.as_slice()).await {
                            return Err(SCError::SSHError(err));
                        }

                        channel_guard.eof().await.unwrap();
                        Ok(SSHMessageResponse::NoOp)
                    };

                    Box::pin(fut.into_actor(self).map(|res, _, _| res))
                } else {
                    Box::pin(async move { Err(SCError::Error) }.into_actor(self).map(
                        |res: Result<SSHMessageResponse, SCError>, _, _| {
                            Ok(SSHMessageResponse::NoOp)
                        },
                    ))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct SSHClient {
    actor_addr: Addr<WSActor>,
}

#[async_trait]
impl client::Handler for SSHClient {
    type Error = anyhow::Error;

    async fn check_server_key(
        self,
        server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        // println!("check_server_key: {:?}", server_public_key);
        Ok((self, true))
    }

    async fn data(
        self,
        channel: ChannelId,
        data: &[u8],
        session: client::Session,
    ) -> Result<(Self, client::Session), Self::Error> {
        // println!(
        //     "data on channel {:?}: {:?}",
        //     channel,
        //     std::str::from_utf8(data)
        // );
        self.actor_addr.do_send(SSHMessage::Data(data.to_owned()));
        Ok((self, session))
    }
}

#[derive(Debug, Clone)]
struct SSHProxyClient;

#[async_trait]
impl client::Handler for SSHProxyClient {
    type Error = anyhow::Error;

    async fn check_server_key(
        self,
        server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        // println!("check_server_key: {:?}", server_public_key);
        Ok((self, true))
    }
}
