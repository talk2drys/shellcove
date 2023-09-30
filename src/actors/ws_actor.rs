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
    ssh_actor: Option<actix::Addr<SSHActor>>,
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

impl Handler<SSHMessage> for WSActor {
    type Result = actix::ResponseActFuture<Self, Result<SSHMessageResponse, SCError>>;

    fn handle(&mut self, msg: SSHMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SSHMessage::Connect {
                host,
                port,
                username,
                password,
                jump_host,
            } => {
                debug!("ssh connection request to {}:{}", host, port);
                let actor_addr = self.ssh_actor.clone().unwrap();
                let request = actor_addr.send(SSHMessage::Connect {
                    host,
                    port,
                    username,
                    password,
                    jump_host,
                });

                let actor_future =
                    request
                        .into_actor(self)
                        .map(|result, _actor, _context| match result {
                            Ok(resp) => {
                                info!("forwarded ssh request to be handled by the ssh actor");
                                resp
                            }
                            Err(_err) => {
                                error!("error forewarding ssh request to ssh_actor");
                                Err::<_, SCError>(SCError::MailBoxError)
                            }
                        });

                // returned the wrapped future
                Box::pin(actor_future)
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
