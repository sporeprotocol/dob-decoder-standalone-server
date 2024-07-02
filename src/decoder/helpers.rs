use std::path::PathBuf;

use ckb_sdk::{constants::TYPE_ID_CODE_HASH, traits::CellQueryOptions};
use ckb_types::{
    core::ScriptHashType,
    packed::{OutPoint, Script},
    prelude::{Builder, Entity, Pack},
    H256,
};
use serde_json::Value;
use spore_types::{generated::spore::ClusterData, SporeData};

use crate::{
    client::RpcClient,
    types::{
        ClusterDescriptionField, DOBDecoderFormat, DecoderLocationType, Error, ScriptId, Settings,
    },
};

pub fn build_type_id_search_option(type_id_args: [u8; 32]) -> CellQueryOptions {
    let type_script = Script::new_builder()
        .code_hash(TYPE_ID_CODE_HASH.0.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(type_id_args.to_vec().pack())
        .build();
    CellQueryOptions::new_type(type_script)
}

pub fn build_batch_search_options(
    type_args: [u8; 32],
    available_script_ids: &[ScriptId],
) -> Vec<CellQueryOptions> {
    available_script_ids
        .iter()
        .map(
            |ScriptId {
                 code_hash,
                 hash_type,
             }| {
                let hash_type: ScriptHashType = hash_type.into();
                let type_script = Script::new_builder()
                    .code_hash(code_hash.0.pack())
                    .hash_type(hash_type.into())
                    .args(type_args.to_vec().pack())
                    .build();
                CellQueryOptions::new_type(type_script)
            },
        )
        .collect()
}

pub fn decode_spore_data(spore_data: &[u8]) -> Result<(Value, String), Error> {
    if spore_data[0] == 0u8 {
        let dna = hex::encode(&spore_data[1..]);
        return Ok((serde_json::Value::String(dna.clone()), dna));
    }

    let value: Value =
        serde_json::from_slice(spore_data).map_err(|_| Error::DOBContentUnexpected)?;
    let dna = match &value {
        serde_json::Value::String(_) => &value,
        serde_json::Value::Array(array) => array.first().ok_or(Error::DOBContentUnexpected)?,
        serde_json::Value::Object(object) => {
            object.get("dna").ok_or(Error::DOBContentUnexpected)?
        }
        _ => return Err(Error::DOBContentUnexpected),
    };
    let dna = match dna {
        serde_json::Value::String(string) => string.to_owned(),
        _ => return Err(Error::DOBContentUnexpected),
    };

    Ok((value, dna))
}

// search on-chain spore cell and return its content field, which represents dob content
pub async fn fetch_dob_content(
    rpc: &RpcClient,
    settings: &Settings,
    spore_id: [u8; 32],
) -> Result<((Value, String), [u8; 32]), Error> {
    let mut spore_cell = None;
    for spore_search_option in build_batch_search_options(spore_id, &settings.available_spores) {
        spore_cell = rpc
            .get_cells(spore_search_option.into(), 1, None)
            .await
            .map_err(|_| Error::FetchLiveCellsError)?
            .objects
            .first()
            .cloned();
        if spore_cell.is_some() {
            break;
        }
    }
    let Some(spore_cell) = spore_cell else {
        return Err(Error::SporeIdNotFound);
    };
    let molecule_spore_data =
        SporeData::from_compatible_slice(spore_cell.output_data.unwrap_or_default().as_bytes())
            .map_err(|_| Error::SporeDataUncompatible)?;
    let content_type = String::from_utf8(molecule_spore_data.content_type().raw_data().to_vec())
        .map_err(|_| Error::SporeDataContentTypeUncompatible)?;
    if !settings
        .protocol_versions
        .iter()
        .any(|version| content_type.starts_with(version))
    {
        return Err(Error::DOBVersionUnexpected);
    }
    let cluster_id = molecule_spore_data
        .cluster_id()
        .to_opt()
        .ok_or(Error::ClusterIdNotSet)?
        .raw_data();
    let dob_content = decode_spore_data(&molecule_spore_data.content().raw_data())?;
    Ok((dob_content, cluster_id.to_vec().try_into().unwrap()))
}

// search on-chain cluster cell and return its description field, which contains dob metadata
pub async fn fetch_dob_metadata(
    rpc: &RpcClient,
    settings: &Settings,
    cluster_id: [u8; 32],
) -> Result<ClusterDescriptionField, Error> {
    let mut cluster_cell = None;
    for cluster_search_option in
        build_batch_search_options(cluster_id, &settings.available_clusters)
    {
        cluster_cell = rpc
            .get_cells(cluster_search_option.into(), 1, None)
            .await
            .map_err(|_| Error::FetchLiveCellsError)?
            .objects
            .first()
            .cloned();
        if cluster_cell.is_some() {
            break;
        }
    }
    let Some(cluster_cell) = cluster_cell else {
        return Err(Error::ClusterIdNotFound);
    };
    let molecule_cluster_data =
        ClusterData::from_compatible_slice(cluster_cell.output_data.unwrap_or_default().as_bytes())
            .map_err(|_| Error::ClusterDataUncompatible)?;
    let dob_metadata = serde_json::from_slice(&molecule_cluster_data.description().raw_data())
        .map_err(|_| Error::DOBMetadataUnexpected)?;
    Ok(dob_metadata)
}

// search on-chain decoder cell, deployed with type_id feature enabled
pub async fn fetch_decoder_binary(rpc: &RpcClient, decoder_id: [u8; 32]) -> Result<Vec<u8>, Error> {
    let decoder_search_option = build_type_id_search_option(decoder_id);
    let decoder_cell = rpc
        .get_cells(decoder_search_option.into(), 1, None)
        .await
        .map_err(|_| Error::FetchLiveCellsError)?
        .objects
        .first()
        .cloned()
        .ok_or(Error::DecoderIdNotFound)?;
    Ok(decoder_cell
        .output_data
        .unwrap_or_default()
        .as_bytes()
        .into())
}

// search on-chain decoder cell, directly by its tx_hash and out_index
pub async fn fetch_decoder_binary_directly(
    rpc: &RpcClient,
    tx_hash: H256,
    out_index: u32,
) -> Result<Vec<u8>, Error> {
    let decoder_cell = rpc
        .get_live_cell(&OutPoint::new(tx_hash.pack(), out_index).into(), true)
        .await
        .map_err(|_| Error::FetchTransactionError)?;
    let decoder_binary = decoder_cell
        .cell
        .ok_or(Error::NoOutputCellInTransaction)?
        .data
        .ok_or(Error::DecoderBinaryNotFoundInCell)?
        .content;
    Ok(decoder_binary.as_bytes().to_vec())
}

pub async fn parse_decoder_path(
    rpc: &RpcClient,
    decoder: &DOBDecoderFormat,
    settings: &Settings,
) -> Result<PathBuf, Error> {
    let decoder_path = match decoder.location {
        DecoderLocationType::CodeHash => {
            let mut decoder_path = settings.decoders_cache_directory.clone();
            decoder_path.push(format!("code_hash_{}.bin", hex::encode(&decoder.hash)));
            if !decoder_path.exists() {
                let onchain_decoder =
                    settings
                        .onchain_decoder_deployment
                        .iter()
                        .find_map(|deployment| {
                            if deployment.code_hash == decoder.hash {
                                Some(fetch_decoder_binary_directly(
                                    rpc,
                                    deployment.tx_hash.clone(),
                                    deployment.out_index,
                                ))
                            } else {
                                None
                            }
                        });
                let Some(decoder_binary) = onchain_decoder else {
                    return Err(Error::NativeDecoderNotFound);
                };
                let decoder_file_content = decoder_binary.await?;
                if ckb_hash::blake2b_256(&decoder_file_content) != decoder.hash.0 {
                    return Err(Error::DecoderBinaryHashInvalid);
                }
                std::fs::write(decoder_path.clone(), decoder_file_content)
                    .map_err(|_| Error::DecoderBinaryPathInvalid)?;
            }
            decoder_path
        }
        DecoderLocationType::TypeId => {
            let mut decoder_path = settings.decoders_cache_directory.clone();
            decoder_path.push(format!("type_id_{}.bin", hex::encode(&decoder.hash)));
            if !decoder_path.exists() {
                let decoder_binary = fetch_decoder_binary(rpc, decoder.hash.clone().into()).await?;
                std::fs::write(decoder_path.clone(), decoder_binary)
                    .map_err(|_| Error::DecoderBinaryPathInvalid)?;
            }
            decoder_path
        }
    };
    Ok(decoder_path)
}
