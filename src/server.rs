use std::sync::Arc;

use jsonrpsee::{proc_macros::rpc, tracing, types::ErrorCode};
use serde::Serialize;

use crate::{decoder::DecoderCmdSender, types::SporeContentField};

// decoding result contains rendered result from native decoder and DNA string for optional use
#[derive(Serialize, Clone)]
pub struct ServerDecodeResult {
    render_output: String,
    dob_content: SporeContentField,
}

#[rpc(server)]
trait DecoderRpc {
    #[method(name = "dob_protocol_version")]
    fn protocol_version(&self) -> String;

    #[method(name = "dob_decode")]
    fn decode(&self, hexed_spore_id: String) -> Result<String, ErrorCode>;
}

pub struct DecoderStandaloneServer {
    sender: Arc<DecoderCmdSender>,
}

impl DecoderStandaloneServer {
    pub fn new(sender: Arc<DecoderCmdSender>) -> Self {
        Self { sender }
    }
}

impl DecoderRpcServer for DecoderStandaloneServer {
    fn protocol_version(&self) -> String {
        self.sender.protocol_version()
    }

    // decode DNA in particular spore DOB cell
    fn decode(&self, hexed_spore_id: String) -> Result<String, ErrorCode> {
        tracing::info!("decoding spore_id {hexed_spore_id}");
        let (render_output, dob_content) = self.sender.decode_dna(&hexed_spore_id)?;
        let result = ServerDecodeResult {
            render_output,
            dob_content,
        };
        Ok(serde_json::to_string(&result).unwrap().replace('\\', ""))
    }
}
