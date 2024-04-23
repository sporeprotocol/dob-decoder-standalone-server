use std::sync::Arc;

use jsonrpsee::core::async_trait;
use jsonrpsee::{proc_macros::rpc, tracing, types::ErrorCode};
use serde::Serialize;

use crate::{decoder::DOBDecoder, types::SporeContentField};

// decoding result contains rendered result from native decoder and DNA string for optional use
#[derive(Serialize, Clone)]
pub struct ServerDecodeResult {
    render_output: String,
    dob_content: SporeContentField,
}

#[rpc(server)]
trait DecoderRpc {
    #[method(name = "dob_protocol_version")]
    async fn protocol_version(&self) -> String;

    #[method(name = "dob_decode")]
    async fn decode(&self, hexed_spore_id: String) -> Result<String, ErrorCode>;
}

pub struct DecoderStandaloneServer {
    decoder: Arc<DOBDecoder>,
}

impl DecoderStandaloneServer {
    pub fn new(decoder: Arc<DOBDecoder>) -> Self {
        Self { decoder }
    }
}

#[async_trait]
impl DecoderRpcServer for DecoderStandaloneServer {
    async fn protocol_version(&self) -> String {
        self.decoder.protocol_version()
    }

    async fn decode(&self, hexed_spore_id: String) -> Result<String, ErrorCode> {
        let spore_id: [u8; 32] = hex::decode(hexed_spore_id).unwrap().try_into().unwrap();
        let decoder = Arc::clone(&self.decoder);
        let ret = tokio::task::spawn_blocking(move || decoder.fetch_dob_content(spore_id))
            .await
            .unwrap()
            .unwrap();
        tracing::info!("spore content: {:?}", ret.0.dna);
        Ok("".into())
    }
}
