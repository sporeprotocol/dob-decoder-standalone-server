use std::path::PathBuf;

use ckb_jsonrpc_types::Script;
use ckb_types::{core::ScriptHashType, H256};
use serde::{ser::SerializeMap, Deserialize};
use serde_json::Value;

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
    #[error("error ocurred while requiring system timestamp")]
    SystemTimeError,
    #[error("no decoder hash fund in DOB metadata")]
    DecoderHashNotFound,
    #[error("no decoder type_script fund in DOB metadata")]
    DecoderScriptNotFound,
    #[error("decoder chain list cannot be empty")]
    DecoderChainIsEmpty,
}

pub enum Dob<'a> {
    V0(&'a DOBClusterFormatV0),
    V1(&'a DOBClusterFormatV1),
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

impl ClusterDescriptionField {
    pub fn unbox_dob(&self) -> Result<Dob, Error> {
        match self.dob.ver {
            Some(0) | None => {
                let dob0 = self
                    .dob
                    .dob_ver_0
                    .as_ref()
                    .ok_or(Error::ClusterDataUncompatible)?;
                Ok(Dob::V0(dob0))
            }
            Some(1) => {
                let dob1 = self
                    .dob
                    .dob_ver_1
                    .as_ref()
                    .ok_or(Error::ClusterDataUncompatible)?;
                Ok(Dob::V1(dob1))
            }
            _ => Err(Error::DOBVersionUnexpected),
        }
    }
}

pub struct DOBSporeFormat {
    pub content_type: String,
    pub content: Value,
    pub dna: String,
    pub cluster_id: [u8; 32],
}

// contains `decoder` and `pattern` identifiers
//
// note: if `ver` is empty, `dob_ver_0` must uniquely exist
#[derive(Deserialize)]
#[cfg_attr(feature = "standalone_server", derive(Serialize))]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct DOBClusterFormat {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ver: Option<u8>,
    #[serde(flatten)]
    pub dob_ver_0: Option<DOBClusterFormatV0>,
    #[serde(flatten)]
    pub dob_ver_1: Option<DOBClusterFormatV1>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "standalone_server", derive(Serialize))]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct DOBClusterFormatV0 {
    pub decoder: DOBDecoderFormat,
    pub pattern: Value,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "standalone_server", derive(Serialize))]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct DOBClusterFormatV1 {
    pub decoders: Vec<DOBClusterFormatV0>,
}

#[cfg(feature = "standalone_server")]
impl DOBClusterFormat {
    #[allow(dead_code)]
    pub fn new_dob0(dob_ver_0: DOBClusterFormatV0) -> Self {
        Self {
            ver: Some(0),
            dob_ver_0: Some(dob_ver_0),
            dob_ver_1: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_dob1(dob_ver_1: DOBClusterFormatV1) -> Self {
        Self {
            ver: Some(1),
            dob_ver_0: None,
            dob_ver_1: Some(dob_ver_1),
        }
    }
}

// restricted decoder locator type
#[derive(Deserialize)]
#[cfg_attr(feature = "standalone_server", derive(Serialize))]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum DecoderLocationType {
    #[serde(rename(serialize = "type_id", deserialize = "type_id"))]
    TypeId,
    #[serde(rename(serialize = "code_hash", deserialize = "code_hash"))]
    CodeHash,
    #[serde(rename(serialize = "type_script", deserialize = "type_script"))]
    TypeScript,
}

// decoder location information
#[derive(Deserialize)]
#[cfg_attr(feature = "standalone_server", derive(Serialize))]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct DOBDecoderFormat {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub location: DecoderLocationType,
    pub hash: Option<H256>,
    // only exists when `location` is `TypeScript`
    pub script: Option<Script>,
}

// asscoiate `code_hash` of decoder binary with its onchain deployment information
#[cfg_attr(feature = "standalone_server", derive(Serialize, Deserialize))]
#[cfg_attr(test, derive(Default))]
pub struct OnchainDecoderDeployment {
    pub code_hash: H256,
    pub tx_hash: H256,
    pub out_index: u32,
}

#[cfg_attr(feature = "standalone_server", derive(Serialize, Deserialize))]
#[cfg_attr(test, derive(Default))]
pub enum HashType {
    #[serde(rename(serialize = "data", deserialize = "data"))]
    #[cfg_attr(test, default)]
    Data,
    #[serde(rename(serialize = "data1", deserialize = "data1"))]
    Data1,
    #[serde(rename(serialize = "data2", deserialize = "data2"))]
    Data2,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    Type,
}

impl From<&HashType> for ScriptHashType {
    fn from(value: &HashType) -> Self {
        match value {
            HashType::Data => ScriptHashType::Data,
            HashType::Data1 => ScriptHashType::Data1,
            HashType::Data2 => ScriptHashType::Data2,
            HashType::Type => ScriptHashType::Type,
        }
    }
}

#[cfg_attr(feature = "standalone_server", derive(Serialize, Deserialize))]
#[cfg_attr(test, derive(Default))]
pub struct ScriptId {
    pub code_hash: H256,
    pub hash_type: HashType,
}

// standalone server settings in TOML format
#[cfg_attr(feature = "standalone_server", derive(Serialize, Deserialize))]
#[cfg_attr(test, derive(Default))]
pub struct Settings {
    pub protocol_versions: Vec<String>,
    pub ckb_rpc: String,
    pub rpc_server_address: String,
    pub decoders_cache_directory: PathBuf,
    pub dobs_cache_directory: PathBuf,
    pub dobs_cache_expiration_sec: u64,
    pub onchain_decoder_deployment: Vec<OnchainDecoderDeployment>,
    pub available_spores: Vec<ScriptId>,
    pub available_clusters: Vec<ScriptId>,
}

#[cfg_attr(feature = "standalone_server", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct StandardDOBOutput {
    pub name: String,
    pub traits: Vec<ParsedTrait>,
}

#[derive(Clone)]
pub struct ParsedTrait {
    pub type_: String,
    pub value: Value,
}

impl Serialize for ParsedTrait {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.type_, &self.value)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for ParsedTrait {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: Value = Deserialize::deserialize(deserializer)?;
        map.as_object()
            .and_then(|map| map.iter().next())
            .map(|(type_, value)| {
                Ok(Self {
                    type_: type_.to_string(),
                    value: value.clone(),
                })
            })
            .unwrap_or_else(|| Err(serde::de::Error::custom("invalid ParsedTrait")))
    }
}
