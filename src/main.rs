mod actors;
mod configuration;
mod constants;
mod error;
mod handler;
mod messages;

use crate::configuration::get_configuration;
use crate::configuration::Setting;
use actix_web::{App, HttpServer};
use handler::{connection, health_check};
use tracing::{error, info};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[actix::main]
async fn main() -> std::io::Result<()> {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");

    let format_layer = fmt::layer();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));
    let subscriber = Registry::default().with(env_filter).with(format_layer);

    tracing::subscriber::set_global_default(subscriber).expect("error setting global subscriber");

    info!("reading configuraion file");
    let config = get_configuration();

    if config.is_err() {
        error!("error getting configuration");
        std::process::exit(10);
    }

    let setting: Setting = config.unwrap(); // would not panic

    info!("starting web server");
    HttpServer::new(|| App::new().service(connection).service(health_check))
        .bind((setting.listen_addr, setting.listen_port))?
        .run()
        .await
}
