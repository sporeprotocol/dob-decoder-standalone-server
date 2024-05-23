use ckb_types::h256;

use crate::types::{OnchainDecoderDeployment, Settings};

mod decoder;
mod legacy_decoder;

fn prepare_settings(version: &str) -> Settings {
    Settings {
        ckb_rpc: "https://testnet.ckbapp.dev/".to_string(),
        protocol_versions: vec![version.to_string()],
        ckb_vm_runner: "ckb-vm-runner".to_string(),
        decoders_cache_directory: "cache/decoders".parse().unwrap(),
        dobs_cache_directory: "cache/dobs".parse().unwrap(),
        avaliable_spore_code_hashes: vec![
            h256!("0x685a60219309029d01310311dba953d67029170ca4848a4ff638e57002130a0d"),
            h256!("0x5e063b4c0e7abeaa6a428df3b693521a3050934cf3b0ae97a800d1bc31449398"),
        ],
        avaliable_cluster_code_hashes: vec![
            h256!("0x0bbe768b519d8ea7b96d58f1182eb7e6ef96c541fbd9526975077ee09f049058"),
            h256!("0x7366a61534fa7c7e6225ecc0d828ea3b5366adec2b58206f2ee84995fe030075"),
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
                    "0xdf2030642f219db0a06f6ee4b160142cc4d668790616b1dc1bdd4e3ff7e3a814"
                ),
                tx_hash: h256!(
                    "0xe5deff07d226fbd88ae9c406b7ba04f6d9f91c0b733b65c78089c55e660a2c1e"
                ),
                out_index: 0,
            },
        ],
        ..Default::default()
    }
}
