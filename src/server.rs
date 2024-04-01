use std::sync::Arc;

use jsonrpsee::{proc_macros::rpc, tracing, types::ErrorCode};

use crate::{decoder::DecoderCmdSender, types::ServerDecodeResult};

#[rpc(server)]
trait DecoderRpc {
    #[method(name = "dob_protocol_version")]
    fn protocol_version(&self) -> String;

    #[method(name = "dob_decode")]
    fn decode(&self, hexed_spore_id: String) -> Result<ServerDecodeResult, ErrorCode>;
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
    fn decode(&self, hexed_spore_id: String) -> Result<ServerDecodeResult, ErrorCode> {
        tracing::info!("decoding spore_id {hexed_spore_id}");
        let (raw_render_result, dob_content) = self.sender.decode_dna(&hexed_spore_id)?;
        Ok(ServerDecodeResult {
            raw_render_result,
            dob_content,
        })
    }
}
