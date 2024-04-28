use std::fs;
use std::path::PathBuf;

use jsonrpsee::core::async_trait;
use jsonrpsee::{proc_macros::rpc, tracing, types::ErrorCode};
use serde::Serialize;

use crate::decoder::DOBDecoder;
use crate::types::{Error, SporeContentField};

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

    #[method(name = "dob_batch_decode")]
    async fn batch_decode(&self, hexed_spore_ids: Vec<String>) -> Result<Vec<String>, ErrorCode>;
}

pub struct DecoderStandaloneServer {
    decoder: DOBDecoder,
}

impl DecoderStandaloneServer {
    pub fn new(decoder: DOBDecoder) -> Self {
        Self { decoder }
    }
}

#[async_trait]
impl DecoderRpcServer for DecoderStandaloneServer {
    async fn protocol_version(&self) -> String {
        self.decoder.protocol_version()
    }

    // decode DNA in particular spore DOB cell
    async fn decode(&self, hexed_spore_id: String) -> Result<String, ErrorCode> {
        tracing::info!("decoding spore_id {hexed_spore_id}");
        let spore_id: [u8; 32] = hex::decode(hexed_spore_id)
            .map_err(|_| Error::HexedSporeIdParseError)?
            .try_into()
            .map_err(|_| Error::SporeIdLengthInvalid)?;
        let mut cache_path = self.decoder.setting().dobs_cache_directory.clone();
        cache_path.push(format!("{}.dob", hex::encode(spore_id)));
        let (render_output, dob_content) = if cache_path.exists() {
            read_dob_from_cache(cache_path)?
        } else {
            let (content, metadata) = self.decoder.fetch_decode_ingredients(spore_id).await?;
            let render_output = self.decoder.decode_dna(&content, metadata).await?;
            write_dob_to_cache(&render_output, &content, cache_path)?;
            (render_output, content)
        };
        let result = ServerDecodeResult {
            render_output,
            dob_content,
        };
        Ok(serde_json::to_string(&result).unwrap())
    }

    // decode DNA from a set
    async fn batch_decode(&self, hexed_spore_ids: Vec<String>) -> Result<Vec<String>, ErrorCode> {
        let mut await_results = Vec::new();
        for hexed_spore_id in hexed_spore_ids {
            await_results.push(self.decode(hexed_spore_id));
        }
        let results = futures::future::join_all(await_results)
            .await
            .into_iter()
            .map(|result| match result {
                Ok(result) => result,
                Err(error) => format!("server error: {error}"),
            })
            .collect();
        Ok(results)
    }
}

pub fn read_dob_from_cache(cache_path: PathBuf) -> Result<(String, SporeContentField), Error> {
    let file_content = fs::read_to_string(cache_path).map_err(|_| Error::DOBRenderCacheNotFound)?;
    let mut lines = file_content.split('\n');
    let (Some(result), Some(content)) = (lines.next(), lines.next()) else {
        return Err(Error::DOBRenderCacheModified);
    };
    match serde_json::from_str::<SporeContentField>(content) {
        Ok(content) => Ok((result.to_string(), content)),
        Err(_) => Err(Error::DOBRenderCacheModified),
    }
}

pub fn write_dob_to_cache(
    render_result: &str,
    dob_content: &SporeContentField,
    cache_path: PathBuf,
) -> Result<(), Error> {
    let json_dob_content = serde_json::to_string(dob_content).unwrap();
    let file_content = format!("{render_result}\n{json_dob_content}");
    fs::write(cache_path, file_content).map_err(|_| Error::DOBRenderCacheNotFound)?;
    Ok(())
}
