use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonrpsee::core::async_trait;
use jsonrpsee::{proc_macros::rpc, tracing, types::ErrorCode};
use serde::Serialize;
use serde_json::Value;

use crate::decoder::helpers::{decode_cluster_data, decode_spore_data};
use crate::decoder::DOBDecoder;
use crate::types::Error;

// decoding result contains rendered result from native decoder and DNA string for optional use
#[derive(Serialize, Clone, Debug)]
pub struct ServerDecodeResult {
    render_output: String,
    dob_content: Value,
}

#[rpc(server)]
trait DecoderRpc {
    #[method(name = "dob_protocol_version")]
    async fn protocol_versions(&self) -> Vec<String>;

    #[method(name = "dob_decode")]
    async fn decode(&self, hexed_spore_id: String) -> Result<String, ErrorCode>;

    #[method(name = "dob_batch_decode")]
    async fn batch_decode(&self, hexed_spore_ids: Vec<String>) -> Result<Vec<String>, ErrorCode>;

    #[method(name = "dob_raw_decode")]
    async fn raw_decode(
        &self,
        spore_data: String,
        cluster_data: String,
    ) -> Result<String, ErrorCode>;
}

pub struct DecoderStandaloneServer {
    decoder: DOBDecoder,
    cache_expiration: u64,
}

impl DecoderStandaloneServer {
    pub fn new(decoder: DOBDecoder, cache_expiration: u64) -> Self {
        Self {
            decoder,
            cache_expiration,
        }
    }

    async fn cache_decode(
        &self,
        spore_id: [u8; 32],
        cache_path: PathBuf,
    ) -> Result<(String, Value), Error> {
        let (content, dna, metadata) = self.decoder.fetch_decode_ingredients(spore_id).await?;
        let render_output = self.decoder.decode_dna(&dna, metadata).await?;
        write_dob_to_cache(&render_output, &content, cache_path, self.cache_expiration)?;
        Ok((render_output, content))
    }
}

#[async_trait]
impl DecoderRpcServer for DecoderStandaloneServer {
    async fn protocol_versions(&self) -> Vec<String> {
        self.decoder.protocol_versions()
    }

    // decode DNA in particular spore DOB cell
    async fn decode(&self, hexed_spore_id: String) -> Result<String, ErrorCode> {
        tracing::info!("decoding spore_id {hexed_spore_id}");
        let spore_id: [u8; 32] = hex::decode(trim_0x(&hexed_spore_id))
            .map_err(|_| Error::HexedSporeIdParseError)?
            .try_into()
            .map_err(|_| Error::SporeIdLengthInvalid)?;
        let mut cache_path = self.decoder.setting().dobs_cache_directory.clone();
        cache_path.push(format!("{}.dob", hex::encode(spore_id)));
        let (render_output, dob_content) =
            if let Some(cache) = read_dob_from_cache(cache_path.clone(), self.cache_expiration)? {
                cache
            } else {
                self.cache_decode(spore_id, cache_path).await?
            };
        let result = serde_json::to_string(&ServerDecodeResult {
            render_output,
            dob_content,
        })
        .unwrap();
        tracing::info!("spore_id {hexed_spore_id}, result: {result}");
        Ok(result)
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

    // decode directly from spore and cluster data
    async fn raw_decode(
        &self,
        hexed_spore_data: String,
        hexed_cluster_data: String,
    ) -> Result<String, ErrorCode> {
        let spore_data =
            hex::decode(trim_0x(&hexed_spore_data)).map_err(|_| Error::SporeDataUncompatible)?;
        let cluster_data = hex::decode(trim_0x(&hexed_cluster_data))
            .map_err(|_| Error::ClusterDataUncompatible)?;
        let dob = decode_spore_data(&spore_data)?;
        let dob_metadata = decode_cluster_data(&cluster_data)?;
        let render_output = self.decoder.decode_dna(&dob.dna, dob_metadata).await?;
        let result = serde_json::to_string(&ServerDecodeResult {
            render_output,
            dob_content: dob.content,
        })
        .unwrap();
        tracing::info!("raw, result: {result}");
        Ok(result)
    }
}

fn trim_0x(hexed: &str) -> &str {
    hexed.trim_start_matches("0x")
}

fn read_dob_from_cache(
    cache_path: PathBuf,
    mut expiration: u64,
) -> Result<Option<(String, Value)>, Error> {
    if !cache_path.exists() {
        return Ok(None);
    }
    let file_content = fs::read_to_string(cache_path).map_err(|_| Error::DOBRenderCacheNotFound)?;
    let mut lines = file_content.split('\n');
    let (Some(result), Some(content), timestamp) = (lines.next(), lines.next(), lines.next())
    else {
        return Err(Error::DOBRenderCacheModified);
    };
    if let Some(value) = timestamp {
        if !value.is_empty() {
            expiration = value
                .parse::<u64>()
                .map_err(|_| Error::DOBRenderCacheModified)?;
        }
    }
    match serde_json::from_str(content) {
        Ok(content) => {
            if expiration > 0 && now()? > Duration::from_secs(expiration) {
                Ok(None)
            } else {
                Ok(Some((result.to_string(), content)))
            }
        }
        Err(_) => Err(Error::DOBRenderCacheModified),
    }
}

fn write_dob_to_cache(
    render_result: &str,
    dob_content: &Value,
    cache_path: PathBuf,
    cache_expiration: u64,
) -> Result<(), Error> {
    let expiration_timestamp = if cache_expiration > 0 {
        now()?
            .checked_add(Duration::from_secs(cache_expiration))
            .ok_or(Error::SystemTimeError)?
            .as_secs()
    } else {
        0 // zero means always read from cache
    };
    let json_dob_content = serde_json::to_string(dob_content).unwrap();
    let file_content = format!("{render_result}\n{json_dob_content}\n{expiration_timestamp}");
    fs::write(cache_path, file_content).map_err(|_| Error::DOBRenderCacheNotFound)?;
    Ok(())
}

fn now() -> Result<Duration, Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::SystemTimeError)
}
