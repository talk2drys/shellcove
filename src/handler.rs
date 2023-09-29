use crate::actors::ws_actor::WSActor;
use actix_web::{get, web, HttpRequest, Responder};
use actix_web_actors::ws;
use tracing::info;

#[get("/ws/")]
pub async fn connection(req: HttpRequest, stream: web::Payload) -> impl Responder {
    let peer_address = req.peer_addr();
    if let Some(addr) = peer_address {
        info!("websocket connection established from peer {}", addr.ip());
    }
    ws::start(WSActor::new(), &req, stream)
}

#[get("/health")]
pub async fn health_check() -> impl Responder {
    println!("it loaded");
    "worked"
}
