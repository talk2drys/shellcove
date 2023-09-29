use super::ssh_actor::SSHActor;
use crate::error::SCError;
use crate::messages::{SSHMessage, SSHMessageResponse};
use actix::ActorFutureExt;
use actix::{Actor, AsyncContext, Handler, ResponseFuture, StreamHandler, WrapFuture};
use actix_web_actors::ws;
use bytestring::ByteString;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct WSActor {
    pub ssh_actor: Option<actix::Addr<SSHActor>>,
}

impl WSActor {
    pub fn new() -> Self {
        WSActor { ssh_actor: None }
    }
}

impl Actor for WSActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // start the ssh actor
        let ws_addr = ctx.address();
        let ssh_actor = SSHActor {
            channel: None,
            ws_addr: Some(ws_addr),
        }
        .start();

        self.ssh_actor = Some(ssh_actor);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WSActor {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => ctx.ping(&msg),
            Ok(ws::Message::Text(text)) => {
                let message: SSHMessage =
                    serde_json::from_str::<SSHMessage>(text.to_string().as_str()).unwrap();
                ctx.notify(message);
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl<'data> Handler<SSHMessage> for WSActor {
    type Result = actix::ResponseActFuture<Self, Result<SSHMessageResponse, SCError>>;

    fn handle(&mut self, msg: SSHMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SSHMessage::Connect {
                host,
                port,
                username,
                password,
                jump_host,
            } => {
                debug!("ssh connection request to {}:{}", host, port);
                let cloned_addr = self.ssh_actor.clone();
                let fut = async move {
                    let addr = cloned_addr.unwrap();
                    let res = addr
                        .send(SSHMessage::Connect {
                            host,
                            port,
                            username,
                            password,
                            jump_host,
                        })
                        .await
                        .unwrap();
                    // match res {
                    //     Ok(resp) => {
                    //         info!("forwarded ssh request to be handled by the ssh actor");
                    //         resp
                    //     }
                    //     Err(err) => {
                    //         error!("error forewarding ssh request to ssh_actor");
                    //         err
                    //     }
                    // }

                    res
                };

                Box::pin(fut.into_actor(self).map(|res, _, _| res))
            }
            SSHMessage::ShellInput(cmd) => {
                let cloned_addr = self.ssh_actor.clone();
                let fut = async move {
                    let addr = cloned_addr.unwrap();
                    let res = addr.send(SSHMessage::ShellInput(cmd)).await;
                    res.unwrap()
                };

                Box::pin(fut.into_actor(self).map(|res, _, _| res))
            }
            SSHMessage::TerminalResize {
                width,
                height,
                pixelwidth,
                pixelheight,
            } => {
                let cloned_addr = self.ssh_actor.clone();
                let fut = async move {
                    let addr = cloned_addr.unwrap();
                    let res = addr
                        .send(SSHMessage::TerminalResize {
                            width,
                            height,
                            pixelwidth,
                            pixelheight,
                        })
                        .await;
                    res.unwrap()
                };
                Box::pin(fut.into_actor(self).map(|res, _, _| res))
            }
            SSHMessage::ShellOutput(data) => {
                let v = data.clone();
                let hh = String::from_utf8(v);
                // dbg!(&hh);
                let fut = async {
                    if let Ok(data) = ByteString::try_from(data) {
                        // dbg!(&data);
                        Ok(data)
                    } else {
                        Err(SCError::ByteParseError)
                    }
                };
                Box::pin(fut.into_actor(self).map(|res, _act, ctx| {
                    ctx.text(res.unwrap());
                    Ok(SSHMessageResponse::NoOp)
                }))
            }
        }
    }
}
