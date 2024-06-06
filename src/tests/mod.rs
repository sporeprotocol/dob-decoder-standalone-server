use ckb_types::h256;

use crate::types::{HashType, OnchainDecoderDeployment, ScriptId, Settings};

mod dob0;
mod dob1;

fn prepare_settings(version: &str) -> Settings {
    Settings {
        ckb_rpc: "https://testnet.ckbapp.dev/".to_string(),
        image_fetcher_url: "https://dobfs.dobby.market/testnet".to_string(),
        protocol_versions: vec![version.to_string()],
        ckb_vm_runner: "ckb-vm-runner".to_string(),
        decoders_cache_directory: "cache/decoders".parse().unwrap(),
        dobs_cache_directory: "cache/dobs".parse().unwrap(),
        dob1_max_combination: 5,
        dob1_max_cache_size: 100,
        available_spores: vec![
            ScriptId {
                code_hash: h256!(
                    "0x685a60219309029d01310311dba953d67029170ca4848a4ff638e57002130a0d"
                ),
                hash_type: HashType::Data1,
            },
            ScriptId {
                code_hash: h256!(
                    "0x5e063b4c0e7abeaa6a428df3b693521a3050934cf3b0ae97a800d1bc31449398"
                ),
                hash_type: HashType::Data1,
            },
        ],
        available_clusters: vec![
            ScriptId {
                code_hash: h256!(
                    "0x0bbe768b519d8ea7b96d58f1182eb7e6ef96c541fbd9526975077ee09f049058"
                ),
                hash_type: HashType::Data1,
            },
            ScriptId {
                code_hash: h256!(
                    "0x7366a61534fa7c7e6225ecc0d828ea3b5366adec2b58206f2ee84995fe030075"
                ),
                hash_type: HashType::Data1,
            },
        ],
        onchain_decoder_deployment: vec![
            OnchainDecoderDeployment {
                code_hash: h256!(
                    "0xb82abd59ade361a014f0abb692f71b0feb880693c3ccb95b9137b73551d872ce"
                ),
                tx_hash: h256!(
                    "0xb2497dc3e616055125ef8276be7ee21986d2cd4b2ce90992725386cabcb6ea7f"
                ),
                out_index: 0,
            },
            OnchainDecoderDeployment {
                code_hash: h256!(
                    "0x32f29aba4b17f3d05bec8cec55d50ef86766fd0bf82fdedaa14269f344d3784a"
                ),
                tx_hash: h256!(
                    "0x987cf95d129a2dcc2cdf7bd387c1bd888fa407e3c5a3d511fd80c80dcf6c6b67"
                ),
                out_index: 0,
            },
        ],
        ..Default::default()
    }
}
