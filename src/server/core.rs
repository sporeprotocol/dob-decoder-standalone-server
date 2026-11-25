use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD, Engine};
use jsonrpsee::core::async_trait;
use jsonrpsee::{proc_macros::rpc, tracing, types::error::ErrorObjectOwned};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::ImageFetchClient;
use crate::decoder::{
    helpers::{decode_cluster_data, decode_spore_data},
    DOBDecoder,
};
use crate::server::utils::{read_dob_from_cache, trim_0x, write_dob_to_cache};
use crate::types::Error;

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

    #[method(name = "dob_extract_image_from_fsuri")]
    async fn extract_image_from_fsuri(
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

    async fn extract_image_from_fsuri(
        &self,
        fsuri: String,
        encode_type: Option<String>,
    ) -> Result<String, ErrorObjectOwned> {
        self.service_extract_image_from_fsuri(fsuri, encode_type)
            .await
    }
}

#[derive(Clone)]
pub struct DecoderStandaloneServer {
    decoder: DOBDecoder,
    cache_expiration: u64,
    image_fetcher: ImageFetchClient,
}

impl DecoderStandaloneServer {
    pub fn new(decoder: DOBDecoder, cache_expiration: u64) -> Self {
        Self {
            image_fetcher: ImageFetchClient::new(&decoder.setting().image_fetcher_url),
            decoder,
            cache_expiration,
        }
    }

    /// Abstracted service method for decoding DOB to JSON
    pub async fn service_decode(&self, hexed_spore_id: String) -> Result<String, ErrorObjectOwned> {
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

    /// Abstracted service method for batch decoding
    pub async fn service_batch_decode(
        &self,
        hexed_spore_ids: Vec<String>,
    ) -> Result<Vec<String>, ErrorObjectOwned> {
        let mut await_results = Vec::new();
        for hexed_spore_id in hexed_spore_ids {
            await_results.push(self.service_decode(hexed_spore_id));
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

    /// Abstracted service method for raw decoding
    pub async fn service_raw_decode(
        &self,
        hexed_spore_data: String,
        hexed_cluster_data: String,
    ) -> Result<String, ErrorObjectOwned> {
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

    /// Abstracted service method for image extraction
    pub async fn service_extract_image_from_fsuri(
        &self,
        fsuri: String,
        encode_type: Option<String>,
    ) -> Result<String, ErrorObjectOwned> {
        let raw_images = self.image_fetcher.fetch_images(&[fsuri]).await?;
        let image = raw_images.first().ok_or(Error::NoImageFound)?;

        match encode_type.as_deref() {
            Some("hex") | None => Ok(hex::encode(image)),
            Some("base64") => Ok(STANDARD.encode(image)),
            unknown => Err(ErrorObjectOwned::owned::<serde_json::Value>(
                -1,
                format!(
                    "Unknown encode type: {}. Supported types: 'base64', 'hex' (default)",
                    unknown.unwrap_or("unknown")
                ),
                None,
            )),
        }
    }

    /// Abstracted service method for protocol versions
    pub async fn service_protocol_versions(&self) -> Vec<String> {
        self.decoder.protocol_versions()
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
