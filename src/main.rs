mod configuration;
use tracing::{debug, info};

#[actix::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::default();
    tracing::subscriber::set_global_default(subscriber).expect("error setting global subscriber");

    info!("reading configuraion file");

    println!("Hello, world!");
}
