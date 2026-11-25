use jsonrpsee::core::async_trait;
use jsonrpsee::{proc_macros::rpc, types::error::ErrorObjectOwned};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::server::DecoderStandaloneServer;

// decoding result contains rendered result from native decoder and DNA string for optional use
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerDecodeResult {
    render_output: String,
    dob_content: Value,
}

#[rpc(server)]
trait DecoderRpc {
    #[method(name = "dob_protocol_version")]
    async fn protocol_versions(&self) -> Vec<String>;

    #[method(name = "dob_decode")]
    async fn decode(&self, hexed_spore_id: String) -> Result<String, ErrorObjectOwned>;

    #[method(name = "dob_batch_decode")]
    async fn batch_decode(
        &self,
        hexed_spore_ids: Vec<String>,
    ) -> Result<Vec<String>, ErrorObjectOwned>;

    #[method(name = "dob_raw_decode")]
    async fn raw_decode(
        &self,
        spore_data: String,
        cluster_data: String,
    ) -> Result<String, ErrorObjectOwned>;

    #[method(name = "dob_extract_image")]
    async fn extract_image(
        &self,
        fsuri: String,
        encode_type: Option<String>,
    ) -> Result<String, ErrorObjectOwned>;
}

#[async_trait]
impl DecoderRpcServer for DecoderStandaloneServer {
    async fn protocol_versions(&self) -> Vec<String> {
        self.service_protocol_versions().await
    }

    // decode DNA in particular spore DOB cell
    async fn decode(&self, hexed_spore_id: String) -> Result<String, ErrorObjectOwned> {
        self.service_decode(hexed_spore_id).await
    }

    // decode DNA from a set
    async fn batch_decode(
        &self,
        hexed_spore_ids: Vec<String>,
    ) -> Result<Vec<String>, ErrorObjectOwned> {
        self.service_batch_decode(hexed_spore_ids).await
    }

    // decode directly from spore and cluster data
    async fn raw_decode(
        &self,
        hexed_spore_data: String,
        hexed_cluster_data: String,
    ) -> Result<String, ErrorObjectOwned> {
        self.service_raw_decode(hexed_spore_data, hexed_cluster_data)
            .await
    }

    async fn extract_image(
        &self,
        fsuri: String,
        encode_type: Option<String>,
    ) -> Result<String, ErrorObjectOwned> {
        self.service_extract_image_from_fsuri(fsuri, encode_type)
            .await
    }
}
