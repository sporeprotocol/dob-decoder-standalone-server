use std::fs;

use jsonrpsee::{server::ServerBuilder, tracing};
use server::DecoderRpcServer;
use tracing_subscriber::EnvFilter;

mod client;
mod decoder;
mod server;
mod types;
mod vm;

const SETTINGS_FILE: &str = "./settings.toml";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("loading settings file from {SETTINGS_FILE}");
    let settings_file = fs::read_to_string(SETTINGS_FILE).expect("read settings.toml");
    let settings: types::Settings = toml::from_str(&settings_file).expect("parse settings.toml");
    tracing::debug!(
        "server settings: {}",
        serde_json::to_string_pretty(&settings).unwrap()
    );
    let rpc_server_address = settings.rpc_server_address.clone();
    let decoder = decoder::DOBDecoder::new(settings);

    tracing::info!("running decoder server at {}", rpc_server_address);
    let http_server = ServerBuilder::new()
        .http_only()
        .build(rpc_server_address)
        .await
        .expect("build http_server");

    let rpc_methods = server::DecoderStandaloneServer::new(decoder);
    let handler = http_server.start(rpc_methods.into_rpc());

    tokio::signal::ctrl_c().await.unwrap();
    tracing::info!("stopping decoder server");
    handler.stop().unwrap();
}
