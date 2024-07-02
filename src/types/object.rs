use std::{collections::HashMap, path::PathBuf};

use ckb_types::{core::ScriptHashType, H256};
use serde::Deserialize;
use serde_json::Value;

#[cfg(feature = "standalone_server")]
use jsonrpsee::types::ErrorCode;
#[cfg(feature = "standalone_server")]
use serde::Serialize;

use crate::types::Error;

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
#[cfg_attr(feature = "standalone_server", derive(Serialize))]
#[cfg_attr(test, derive(PartialEq, Debug))]
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

#[cfg(test)]
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
    pub traits: DOBClusterFormatV0,
    pub images: DOBClusterFormatV0,
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
}

// decoder location information
#[derive(Deserialize)]
#[cfg_attr(feature = "standalone_server", derive(Serialize))]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct DOBDecoderFormat {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub location: DecoderLocationType,
    pub hash: H256,
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
    fn from(hash_type: &HashType) -> ScriptHashType {
        match hash_type {
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
    pub image_fetcher_url: HashMap<String, String>,
    pub rpc_server_address: String,
    pub ckb_vm_runner: String,
    pub decoders_cache_directory: PathBuf,
    pub dobs_cache_directory: PathBuf,
    pub dobs_cache_expiration_sec: u64,
    pub dob1_max_combination: usize,
    pub dob1_max_cache_size: usize,
    pub onchain_decoder_deployment: Vec<OnchainDecoderDeployment>,
    pub available_spores: Vec<ScriptId>,
    pub available_clusters: Vec<ScriptId>,
}
