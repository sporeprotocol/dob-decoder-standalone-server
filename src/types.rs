use std::path::PathBuf;

use ckb_jsonrpc_types::Script;
use ckb_types::{core::ScriptHashType, H256};
use serde::{ser::SerializeMap, Deserialize};
use serde_json::Value;

#[cfg(feature = "standalone_server")]
use jsonrpsee::types::ErrorObjectOwned;
#[cfg(feature = "standalone_server")]
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("DOB version number only support 0 and 1, please check your cluster config")]
    DOBVersionNumberUndefined,
    #[error("spore id byte length should be equal to 32")]
    SporeIdLengthInvalid,
    #[error("decoder of spore dob is not set into server's configuration")]
    NativeDecoderNotFound,
    #[error("spore id not found from ckb network")]
    SporeIdNotFound(String),
    #[error("spore data is not in format of spore protocol")]
    SporeDataUncompatible,
    #[error("content type of spore is not in utf-8 format")]
    SporeDataContentTypeUncompatible,
    #[error("declared dob version `{0}` isn't set into server's configuration")]
    DOBVersionUnexpected(String),
    #[error("cluster id not found in spore data")]
    ClusterIdNotSet,
    #[error("cluster id `{0}` not found from ckb network")]
    ClusterIdNotFound(String),
    #[error("cluster data is not in format of spore protocol")]
    ClusterDataUncompatible,
    #[error("decoder script configured in cluster not found from ckb network")]
    DecoderIdNotFound,
    #[error("output of decoder program not in format json")]
    DecoderOutputInvalid,
    #[error("decoder program outputs nothing, please check decoder program code")]
    DecoderOutputEmpty,
    #[error("spore id string is not in hex format")]
    HexedSporeIdParseError,
    #[error("configured decoder binary persistence path is unwriteable")]
    DecoderBinaryPathInvalid,
    #[error("execute_riscv_binary call failed: {0}")]
    DecoderExecutionError(String),
    #[error("decoder program triggered an error code: {0}")]
    DecoderExecutionInternalError(i8),
    #[error("get_cells or get_live_cell rpc failed: {0}")]
    FetchLiveCellsError(String),
    #[error("get_transaction or get_transactions rpc failed: {0}")]
    FetchTransactionError(String),
    #[error("decoder cell not found in outpoint({0}:{1})")]
    DecoderCellNotFound(String, u32),
    #[error("spore content doesn't follow the specs of DOB protocol")]
    DOBContentUnexpected,
    #[error("cluster description doesn't follow the specs of DOB protocol")]
    DOBMetadataUnexpected,
    #[error("configured DOB render cache file `{0}` is unwriteable or unreadable")]
    DOBRenderCacheNotFound(PathBuf),
    #[error("DOB render cache file `{0}` has been mannually modified")]
    DOBRenderCacheModified(PathBuf),
    #[error("cached decoder binary file `{0}` has modified")]
    DecoderBinaryHashInvalid(PathBuf),
    #[error("deployed decoder cell has empty cell data")]
    DecoderBinaryNotFoundInCell,
    #[error("JSON-RPC requesting error: {0}")]
    JsonRpcRequestError(String),
    #[error("system time calculation error")]
    SystemTimeError,
    #[error("decoder in cluster used code_hash or type_id type, but no `hash` field found")]
    DecoderHashNotFound,
    #[error("decoder in cluster used type_script type, but no `script` field found")]
    DecoderScriptNotFound,
    #[error("decoders configured in cluster are empty, please check your cluster config")]
    DecoderChainIsEmpty,
}

pub enum Dob<'a> {
    V0(&'a DOBClusterFormatV0),
    V1(&'a DOBClusterFormatV1),
}

#[cfg(feature = "standalone_server")]
impl From<Error> for ErrorObjectOwned {
    fn from(value: Error) -> Self {
        let message = value.to_string();
        Self::owned::<serde_json::Value>(-1, message, None)
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
            _ => Err(Error::DOBVersionNumberUndefined),
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
