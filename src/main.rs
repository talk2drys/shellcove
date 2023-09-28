mod configuration;
mod error;

use crate::configuration::get_configuration;
use crate::configuration::Setting;
use actix_web::{App, HttpServer};
use tracing::{error, info};

#[actix::main]
async fn main() -> std::io::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::default();
    tracing::subscriber::set_global_default(subscriber).expect("error setting global subscriber");

    info!("reading configuraion file");
    let config = get_configuration();

    if config.is_err() {
        error!("error getting configuration");
        std::process::exit(10);
    }

    let setting: Setting = config.unwrap(); // would not panic

    info!("starting web server");
    HttpServer::new(|| App::new())
        .bind((setting.listen_addr, setting.listen_port))?
        .run()
        .await
}
