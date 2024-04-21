pub mod decoder;
pub mod types;

#[cfg(feature = "embeded_vm")]
mod vm;

#[cfg(test)]
mod test {
    use ckb_types::{h256, H256};

    use crate::decoder::{DOBDecoder, DOBThreadDecoder};
    use crate::types::{
        ClusterDescriptionField, DOBClusterFormat, DOBDecoderFormat, DecoderLocationType, Settings,
        SporeContentField,
    };

    const EXPECTED_RENDER_RESULT: &str = "[{\"name\":\"horn\",\"traits\":[{\"String\":\"Gold\"}]},{\"name\":\"wings\",\"traits\":[{\"String\":\"Colorful\"}]},{\"name\":\"body\",\"traits\":[{\"String\":\"White Water\"}]},{\"name\":\"tail\",\"traits\":[{\"String\":\"Colorful\"}]},{\"name\":\"hair\",\"traits\":[{\"String\":\"Yang-Short\"}]},{\"name\":\"horseshoes\",\"traits\":[{\"String\":\"White\"}]},{\"name\":\"talent\",\"traits\":[{\"String\":\"Crown\"}]},{\"name\":\"hp\",\"traits\":[{\"Number\":59576}]},{\"name\":\"lucky\",\"traits\":[{\"Number\":3}]}]";
    const HEXED_SPORE_ID: &str = "0e61cc8eb420ce4ae44c922bfd17bc12204cf95f017a030f5a108d01339feb78";
    const SPORE_ID: H256 =
        h256!("0x0e61cc8eb420ce4ae44c922bfd17bc12204cf95f017a030f5a108d01339feb78");

    fn prepare_settings(version: &str) -> Settings {
        Settings {
            ckb_rpc: "https://testnet.ckbapp.dev/".to_string(),
            protocol_version: version.to_string(),
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
            ..Default::default()
        }
    }

    fn generate_dob_ingredients(
        onchain_decoder: bool,
    ) -> (SporeContentField, ClusterDescriptionField) {
        let unicorn_content = SporeContentField {
            id: 145,
            dna: "44f24502b672369f94808892".to_string(),
        };
        let decoder = if onchain_decoder {
            DOBDecoderFormat {
                location: DecoderLocationType::TypeId,
                hash: h256!("0x11b80e7161c4eed1101e52a5835e9f58008334bc75d5561fefcc62ccc56221cf"),
            }
        } else {
            DOBDecoderFormat {
                location: DecoderLocationType::CodeHash,
                hash: h256!("0xedbb2d19515ebbf69be66b2178b0c4c0884fdb33878bd04a5ad68736a6af74f8"),
            }
        };
        let unicorn_metadata = ClusterDescriptionField {
            description: "Unicorn Collection".to_string(),
            dob: DOBClusterFormat {
                ver: 0,
                decoder,
                pattern: "0a04000028000000990000000b01000098010000090200005c020000d302000091030000cc030000710000000c0000001400000004000000686f726e5d00000008000000550000000c0000000d000000010000000044000000180000002000000027000000330000003b00000004000000426c75650300000052656408000000436f6c6f7266756c04000000476f6c64050000005768697465720000000c000000150000000500000077696e67735d00000008000000550000000c0000000d000000010000000044000000180000002000000027000000330000003b00000004000000426c75650300000052656408000000436f6c6f7266756c04000000476f6c640500000057686974658d0000000c0000001400000004000000626f64797900000008000000710000000c0000000d000000010000000060000000180000002500000031000000430000005100000009000000426c756520576f6f640800000052656420466972650e000000436f6c6f7266756c2045617274680a000000476f6c64204d6574616c0b0000005768697465205761746572710000000c00000014000000040000007461696c5d00000008000000550000000c0000000d000000010000000044000000180000002000000027000000330000003b00000004000000426c75650300000052656408000000436f6c6f7266756c04000000476f6c64050000005768697465530000000c0000001400000004000000686169723f00000008000000370000000c0000000d0000000100000000260000000c0000001a0000000a00000059616e672d53686f72740800000059696e2d4c6f6e67770000000c0000001a0000000a000000686f72736573686f65735d00000008000000550000000c0000000d000000010000000044000000180000002000000027000000330000003b00000004000000426c75650300000052656408000000436f6c6f7266756c04000000476f6c64050000005768697465be0000000c000000160000000600000074616c656e74a800000008000000a00000000c0000000d00000001000000008f0000002c00000037000000400000004b000000540000005d00000067000000700000007a00000085000000070000005265766976616c0500000044656174680700000050726f706865740500000043757273650500000043726f776e060000004865726d69740500000047756172640600000041747461636b0700000043616c6c696e6706000000466f726765743b0000000c000000120000000200000068702900000008000000210000000c0000000d000000040300000050c3000000000000a0860100000000003e0000000c00000015000000050000006c75636b792900000008000000210000000c0000000d000000010300000001000000000000003100000000000000".to_string(),
                dna_bytes: 12,
            },
        };
        (unicorn_content, unicorn_metadata)
    }

    fn decode_unicorn_dna(onchain_decoder: bool) -> String {
        let settings = prepare_settings("text/plain");
        let decoder = DOBDecoder::new(settings);
        let (unicorn_content, unicorn_metadata) = generate_dob_ingredients(onchain_decoder);
        decoder
            .decode_dna(&unicorn_content, unicorn_metadata)
            .expect("decode")
    }

    #[test]
    fn test_decode_unicorn_dna() {
        let render_result = decode_unicorn_dna(false);
        assert_eq!(render_result, EXPECTED_RENDER_RESULT);
    }

    #[test]
    fn test_decode_unicorn_dna_with_onchain_decoder() {
        let render_result = decode_unicorn_dna(true);
        assert_eq!(render_result, EXPECTED_RENDER_RESULT);
    }

    #[should_panic = "fetch: DOBContentUnexpected"]
    #[test]
    fn test_fetch_and_decode_unicorn_dna() {
        let settings = prepare_settings("text/plain");
        let decoder = DOBDecoder::new(settings);
        let (dob_content, dob_metadata) = decoder
            .fetch_decode_ingredients(SPORE_ID.into())
            .expect("fetch");
        let render_result = decoder
            .decode_dna(&dob_content, dob_metadata)
            .expect("decode");
        assert_eq!(render_result, EXPECTED_RENDER_RESULT);
    }

    #[test]
    #[should_panic = "fetch: DOBVersionUnexpected"]
    fn test_fetch_onchain_dob_failed() {
        let settings = prepare_settings("dob/0");
        DOBDecoder::new(settings)
            .fetch_decode_ingredients(SPORE_ID.into())
            .expect("fetch");
    }

    #[should_panic = "thread decode: DOBRenderCacheModifie"]
    #[test]
    fn test_decode_onchain_unicorn_dna_in_thread() {
        let protocol_version = "text/plain";
        let settings = prepare_settings(protocol_version);
        let (decoder, cmd) = DOBThreadDecoder::new(settings);
        decoder.run();
        assert_eq!(cmd.protocol_version(), protocol_version);
        let (render_result, _) = cmd.decode_dna(HEXED_SPORE_ID).expect("thread decode");
        assert_eq!(render_result, EXPECTED_RENDER_RESULT);
        cmd.stop();
    }

    #[test]
    fn test_json_serde() {
        let (unicorn_content, unicorn_metadata) = generate_dob_ingredients(true);
        let json_unicorn_content = serde_json::to_string(&unicorn_content).unwrap();
        let json_unicorn_metadata = serde_json::to_string(&unicorn_metadata).unwrap();
        println!("[spore_content] = {json_unicorn_content}");
        println!("[cluster_description] = {json_unicorn_metadata}");
        let deser_unicorn_content: SporeContentField =
            serde_json::from_slice(json_unicorn_content.as_bytes()).unwrap();
        let deser_unicorn_metadata: ClusterDescriptionField =
            serde_json::from_slice(json_unicorn_metadata.as_bytes()).unwrap();
        assert_eq!(unicorn_content, deser_unicorn_content);
        assert_eq!(unicorn_metadata, deser_unicorn_metadata);
    }
}
