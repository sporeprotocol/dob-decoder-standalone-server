use std::path::PathBuf;

use ckb_types::H256;
use serde::Deserialize;

#[cfg(feature = "standalone_server")]
use jsonrpsee::types::ErrorCode;
#[cfg(feature = "standalone_server")]
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
#[repr(i32)]
pub enum Error {
    #[error("DNA bytes length not match the requirement in Cluster")]
    DnaLengthNotMatch = 1001,
    #[error("spore id length should equal to 32")]
    SporeIdLengthInvalid,
    #[error("natvie decoder not found")]
    NativeDecoderNotFound,
    #[error("spore id not exist on-chain")]
    SporeIdNotFound,
    #[error("uncompatible spore data")]
    SporeDataUncompatible,
    #[error("uncompatible spore data content_type")]
    SporeDataContentTypeUncompatible,
    #[error("unexpected DOB protocol version")]
    DOBVersionUnexpected,
    #[error("miss cluster id in spore data")]
    ClusterIdNotSet,
    #[error("cluster id not exist on-chain")]
    ClusterIdNotFound,
    #[error("uncompatible cluster data")]
    ClusterDataUncompatible,
    #[error("decoder id not exist on-chain")]
    DecoderIdNotFound,
    #[error("output of decoder should contain at least one line")]
    DecoderOutputInvalid,
    #[error("DNA string is not in hex format")]
    HexedDNAParseError,
    #[error("spore id string is not in hex format")]
    HexedSporeIdParseError,
    #[error("invalid decoder path to persist")]
    DecoderBinaryPathInvalid,
    #[error("encounter error while executing DNA decoding")]
    DecoderExecutionError,
    #[error("decoding program triggered an error")]
    DecoderExecutionInternalError,
    #[error("encounter error while searching live cells")]
    FetchLiveCellsError,
    #[error("encounter error while searching transaction by hash")]
    FetchTransactionError,
    #[error("not found specific output_cell in transaction")]
    NoOutputCellInTransaction,
    #[error("spore content cannot parse to DOB content")]
    DOBContentUnexpected,
    #[error("cluster description cannot parse to DOB metadata")]
    DOBMetadataUnexpected,
    #[error("DOB render cache folder not found")]
    DOBRenderCacheNotFound,
    #[error("cached DOB render result file has changed unexpectedly")]
    DOBRenderCacheModified,
    #[error("invalid deployed on-chain decoder code_hash")]
    DecoderBinaryHashInvalid,
    #[error("no binary found in cell for decoder")]
    DecoderBinaryNotFoundInCell,
    #[error("error ocurred while requesing json-rpc")]
    JsonRpcRequestError,
}

#[cfg(feature = "standalone_server")]
impl From<Error> for ErrorCode {
    fn from(value: Error) -> Self {
        (value as i32).into()
    }
}

// value on `description` field in Cluster data, adapting for DOB protocol in JSON format
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize, PartialEq, Debug))]
pub struct ClusterDescriptionField {
    pub description: String,
    pub dob: DOBClusterFormat,
}

// contains `decoder` and `pattern` identifiers
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize, PartialEq, Debug))]
pub struct DOBClusterFormat {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ver: Option<u8>,
    pub decoder: DOBDecoderFormat,
    pub pattern: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dna_bytes: Option<u8>,
}

// restricted decoder locator type
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize, PartialEq, Debug))]
pub enum DecoderLocationType {
    #[serde(rename(serialize = "type_id", deserialize = "type_id"))]
    TypeId,
    #[serde(rename(serialize = "code_hash", deserialize = "code_hash"))]
    CodeHash,
}

// decoder location information
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize, PartialEq, Debug))]
pub struct DOBDecoderFormat {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub location: DecoderLocationType,
    pub hash: H256,
}

// value on `content` field in Spore data, adapting for DOB protocol in JSON format
#[derive(Deserialize)]
#[cfg_attr(feature = "standalone_server", derive(Serialize, Clone, Debug))]
#[cfg_attr(test, derive(PartialEq))]
pub struct SporeContentFieldObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u16>,
    pub dna: String,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "standalone_server", derive(Serialize, Clone, Debug))]
#[cfg_attr(test, derive(PartialEq))]
pub enum SporeContentField {
    Object(SporeContentFieldObject),
    String(String),
    Array(Vec<String>),
}

impl SporeContentField {
    pub fn dna_set(&self) -> Vec<&str> {
        match self {
            SporeContentField::Object(val) => vec![&val.dna],
            SporeContentField::String(val) => vec![&val],
            SporeContentField::Array(val) => val.iter().map(|x| x.as_str()).collect(),
        }
    }
}

// asscoiate `code_hash` of decoder binary with its onchain deployment information
#[cfg_attr(feature = "standalone_server", derive(Serialize, Deserialize))]
#[cfg_attr(test, derive(Default))]
pub struct OnchainDecoderDeployment {
    pub code_hash: H256,
    pub tx_hash: H256,
    pub out_index: u32,
}

// standalone server settings in TOML format
#[cfg_attr(feature = "standalone_server", derive(Serialize, Deserialize))]
#[cfg_attr(test, derive(Default))]
pub struct Settings {
    pub protocol_versions: Vec<String>,
    pub ckb_rpc: String,
    pub rpc_server_address: String,
    pub ckb_vm_runner: String,
    pub decoders_cache_directory: PathBuf,
    pub dobs_cache_directory: PathBuf,
    pub onchain_decoder_deployment: Vec<OnchainDecoderDeployment>,
    pub avaliable_spore_code_hashes: Vec<H256>,
    pub avaliable_cluster_code_hashes: Vec<H256>,
}
