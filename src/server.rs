use std::sync::{
    mpsc::{channel, Sender},
    Arc,
};

use jsonrpsee::{proc_macros::rpc, tracing, types::ErrorCode};

use crate::{
    decoder::DecoderCommand,
    types::{Error, ServerDecodeResult},
};

#[rpc(server)]
trait DecoderRpc {
    #[method(name = "dob_protocol_version")]
    fn protocol_version(&self) -> String;

    #[method(name = "dob_decode")]
    fn decode(&self, hexed_spore_id: String) -> Result<ServerDecodeResult, ErrorCode>;
}

pub struct DecoderStandaloneServer {
    sender: Arc<Sender<DecoderCommand>>,
}

impl DecoderStandaloneServer {
    pub fn new(sender: Arc<Sender<DecoderCommand>>) -> Self {
        Self { sender }
    }
}

impl DecoderRpcServer for DecoderStandaloneServer {
    fn protocol_version(&self) -> String {
        let (tx, rx) = channel();
        self.sender
            .send(DecoderCommand::ProtocolVersion(tx))
            .unwrap();
        rx.recv().unwrap()
    }

    // decode DNA in particular spore DOB cell
    fn decode(&self, hexed_spore_id: String) -> Result<ServerDecodeResult, ErrorCode> {
        tracing::info!("decoding spore_id {hexed_spore_id}");
        let spore_id: [u8; 32] = hex::decode(hexed_spore_id)
            .map_err(|_| Error::HexedSporeIdParseError)?
            .try_into()
            .map_err(|_| Error::SporeIdLengthInvalid)?;
        let (tx, rx) = channel();
        self.sender
            .send(DecoderCommand::DecodeDNA(spore_id, tx))
            .unwrap();
        let (raw_render_result, dob_content) = rx.recv().unwrap()?;
        Ok(ServerDecodeResult {
            raw_render_result,
            dob_content,
        })
    }
}
