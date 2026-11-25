use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use jsonrpsee::tracing;
use serde::Deserialize;

use crate::server::DecoderStandaloneServer;

#[derive(Deserialize)]
struct ExtractImageQuery {
    uri: String,
    encode: Option<String>,
}

// RESTful API implementation
impl DecoderStandaloneServer {
    /// Create RESTful API routes
    pub fn create_restful_routes() -> Router<Self> {
        Router::new()
            .route("/dob_decode/:spore_id", get(handle_dob_decode))
            .route("/dob_batch_decode/:spore_ids", get(handle_dob_batch_decode))
            .route("/dob_extract_image", get(handle_extract_image_from_fsuri))
            .route("/protocol_versions", get(handle_protocol_versions))
    }
}

/// Handle dob_decode RESTful endpoint
async fn handle_dob_decode(
    Path(spore_id): Path<String>,
    State(server): State<DecoderStandaloneServer>,
) -> impl IntoResponse {
    tracing::info!("RESTful API: decoding spore_id {}", spore_id);

    match server.service_decode(spore_id).await {
        Ok(result) => {
            tracing::info!("RESTful API: decode successful");
            (StatusCode::OK, Html(result))
        }
        Err(error) => {
            tracing::error!("RESTful API: decode failed: {}", error);
            (
                StatusCode::BAD_REQUEST,
                Html(format!("Error: {}", error.message())),
            )
        }
    }
}

/// Handle dob_batch_decode RESTful endpoint
async fn handle_dob_batch_decode(
    Path(spore_ids): Path<String>,
    State(server): State<DecoderStandaloneServer>,
) -> impl IntoResponse {
    tracing::info!("RESTful API: batch decoding spore_ids: {}", spore_ids);

    // Parse comma-separated spore IDs
    let spore_id_list: Vec<String> = spore_ids.split(',').map(|s| s.trim().to_string()).collect();

    match server.service_batch_decode(spore_id_list).await {
        Ok(results) => {
            tracing::info!("RESTful API: batch decode successful");
            let json_result = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
            (StatusCode::OK, Html(json_result))
        }
        Err(error) => {
            tracing::error!("RESTful API: batch decode failed: {}", error);
            (
                StatusCode::BAD_REQUEST,
                Html(format!("Error: {}", error.message())),
            )
        }
    }
}

/// Handle dob_extract_image_from_fsuri RESTful endpoint
async fn handle_extract_image_from_fsuri(
    Query(query): Query<ExtractImageQuery>,
    State(server): State<DecoderStandaloneServer>,
) -> impl IntoResponse {
    tracing::info!(
        "RESTful API: extracting image from fsuri: {} with encode: {:?}",
        query.uri,
        query.encode
    );

    match server
        .service_extract_image_from_fsuri(query.uri, query.encode)
        .await
    {
        Ok(result) => {
            tracing::info!("RESTful API: image extraction successful");
            (StatusCode::OK, Html(result))
        }
        Err(error) => {
            tracing::error!("RESTful API: image extraction failed: {}", error);
            (
                StatusCode::BAD_REQUEST,
                Html(format!("Error: {}", error.message())),
            )
        }
    }
}

/// Handle protocol_versions RESTful endpoint
async fn handle_protocol_versions(
    State(server): State<DecoderStandaloneServer>,
) -> impl IntoResponse {
    tracing::info!("RESTful API: getting protocol versions");

    let versions = server.service_protocol_versions().await;
    let json_result = serde_json::to_string(&versions).unwrap_or_else(|_| "[]".to_string());

    tracing::info!("RESTful API: protocol versions retrieved successfully");
    (StatusCode::OK, Html(json_result))
}
