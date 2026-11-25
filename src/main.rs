use std::fs;

use jsonrpsee::{server::Server, tracing};
use server::DecoderRpcServer;
use tower_http::cors::{Any, CorsLayer};
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

    tracing::info!("ensuring cache directories exist");
    fs::create_dir_all(&settings.decoders_cache_directory)
        .expect("failed to create decoders cache directory");
    fs::create_dir_all(&settings.dobs_cache_directory)
        .expect("failed to create DOBs cache directory");
    tracing::info!(
        "decoders cache directory: {:?}",
        settings.decoders_cache_directory
    );
    tracing::info!("DOBs cache directory: {:?}", settings.dobs_cache_directory);

    let rpc_server_address = settings.rpc_server_address.clone();
    let restful_server_address = settings.restful_server_address.clone();
    let cache_expiration = settings.dobs_cache_expiration_sec;
    let decoder = decoder::DOBDecoder::new(settings);

    // Create the decoder server instance
    let decoder_server = server::DecoderStandaloneServer::new(decoder, cache_expiration);
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Start JSON-RPC server
    tracing::info!("running JSON-RPC decoder server at {}", rpc_server_address);
    let http_middleware = tower::ServiceBuilder::new().layer(cors.clone());
    let http_server = Server::builder()
        .set_http_middleware(http_middleware)
        .build(rpc_server_address.clone())
        .await
        .expect("build http_server");

    let rpc_handler = http_server.start(decoder_server.clone().into_rpc());

    // Start RESTful API server
    let restful_handle = if let Some(restful_server_address) = restful_server_address {
        tracing::info!("running RESTful API server at {}", restful_server_address);

        let app = server::DecoderStandaloneServer::create_restful_routes()
            .with_state(decoder_server)
            .layer(cors);

        let restful_listener = tokio::net::TcpListener::bind(&restful_server_address)
            .await
            .expect("Failed to bind RESTful server");

        let restful_handle = tokio::spawn(async move {
            axum::serve(restful_listener, app)
                .await
                .expect("RESTful server failed");
        });

        Some(restful_handle)
    } else {
        None
    };

    tokio::signal::ctrl_c().await.unwrap();
    tracing::info!("stopping both servers");

    rpc_handler.stop().unwrap();
    if let Some(restful_handle) = restful_handle {
        restful_handle.abort();
    }
}
