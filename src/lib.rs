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

    const EXPECTED_UNICORN_RENDER_RESULT: &str = "[{\"name\":\"Spirits\",\"traits\":[{\"String\":\"Metal, Golden Body\"}]},{\"name\":\"Yin Yang\",\"traits\":[{\"String\":\"Yang, Short Hair\"}]},{\"name\":\"Talents\",\"traits\":[{\"String\":\"Forget\"}]},{\"name\":\"Horn\",\"traits\":[{\"String\":\"Necromancer Horn\"}]},{\"name\":\"Wings\",\"traits\":[{\"String\":\"Lightning Wings\"}]},{\"name\":\"Tails\",\"traits\":[{\"String\":\"Dumbledore Tails\"}]},{\"name\":\"Horseshoes\",\"traits\":[{\"String\":\"Colorful Stone Horseshoes\"}]},{\"name\":\"Destiny Number\",\"traits\":[{\"Number\":59576}]},{\"name\":\"Lucky Number\",\"traits\":[{\"Number\":3}]}]";
    const EXPECTED_NERVAPE_RENDER_RESULT: &str = "[{\"name\":\"prev.type\",\"traits\":[{\"String\":\"text\"}]},{\"name\":\"prev.bg\",\"traits\":[{\"String\":\"btcfs://59e87ca177ef0fd457e87e9f93627660022cf519b531e1f4e3a6dda9e5e33827i0\"}]},{\"name\":\"prev.bgcolor\",\"traits\":[{\"String\":\"#CEBAF7\"}]},{\"name\":\"Background\",\"traits\":[{\"Number\":170}]},{\"name\":\"Suit\",\"traits\":[{\"Number\":236}]},{\"name\":\"Upper body\",\"traits\":[{\"Number\":53}]},{\"name\":\"Lower body\",\"traits\":[{\"Number\":189}]},{\"name\":\"Headwear\",\"traits\":[{\"Number\":175}]},{\"name\":\"Mask\",\"traits\":[{\"Number\":153}]},{\"name\":\"Eyewear\",\"traits\":[{\"Number\":126}]},{\"name\":\"Mouth\",\"traits\":[{\"Number\":14}]},{\"name\":\"Ears\",\"traits\":[{\"Number\":165}]},{\"name\":\"Tattoo\",\"traits\":[{\"Number\":231}]},{\"name\":\"Accessory\",\"traits\":[{\"Number\":78}]},{\"name\":\"Handheld\",\"traits\":[{\"Number\":240}]},{\"name\":\"Special\",\"traits\":[{\"Number\":70}]}]";
    const NERVAPE_SPORE_ID: H256 =
        h256!("0x9dd9604d44d6640d1533c9f97f89438f17526e645f6c35aa08d8c7d844578580");
    const UNICORN_SPORE_ID: H256 =
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

    fn generate_nervape_dob_ingredients(
        onchain_decoder: bool,
    ) -> (SporeContentField, ClusterDescriptionField) {
        let nervape_content = SporeContentField {
            id: Some(145),
            dna: "cde3feaaec35bdaf997e0ea5e74ef046".to_string(),
            block_number: None,
            cell_id: None,
        };
        let decoder = if onchain_decoder {
            DOBDecoderFormat {
                location: DecoderLocationType::TypeId,
                hash: h256!("0x11b80e7161c4eed1101e52a5835e9f58008334bc75d5561fefcc62ccc56221cf"),
            }
        } else {
            DOBDecoderFormat {
                location: DecoderLocationType::CodeHash,
                hash: h256!("0xb82abd59ade361a014f0abb692f71b0feb880693c3ccb95b9137b73551d872ce"),
            }
        };
        let nervape_metadata = ClusterDescriptionField {
            description: "Nervape, multi-chain composable digital objects built on Bitcoin.".to_string(),
            dob: DOBClusterFormat {
                ver: Some(0),
                decoder,
                pattern: "830900004400000087000000370500004206000085060000c2060000050700004807000089070000c6070000060800004408000081080000c00800000209000043090000430000000c0000001900000009000000707265762e747970652a00000008000000220000000c0000000d0000000100000000110000000800000005000000696d616765b00400000c0000001700000007000000707265762e62679904000008000000910400000c0000000d0000000100000000800400003c0000008a000000d80000002601000074010000c2010000100200005e020000ac020000fa0200004803000096030000e4030000320400004a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569304a00000062746366733a2f2f6264303863363664636166366165316461656664646166393032386132353539653365396363353866383836336561303761393930613533313037383861653569300b0100000c0000001c0000000c000000707265762e6267636f6c6f72ef00000008000000e70000000c0000000d0000000100000000d60000003c00000047000000520000005d00000068000000730000007e00000089000000940000009f000000aa000000b5000000c0000000cb00000007000000234646453345420700000023464642444643070000002344344330464607000000234146453746390700000023414246344430070000002345384541424507000000234643463841430700000023454142433842070000002346464438383007000000234646453243370700000023464642353744070000002346464144414207000000234530453145320700000023413341374141430000000c0000001a0000000a0000004261636b67726f756e642900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003d0000000c0000001400000004000000537569742900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000430000000c0000001a0000000a000000557070657220626f64792900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000430000000c0000001a0000000a0000004c6f77657220626f64792900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000410000000c000000180000000800000048656164776561722900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003d0000000c00000014000000040000004d61736b2900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000400000000c0000001700000007000000457965776561722900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003e0000000c00000015000000050000004d6f7574682900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003d0000000c0000001400000004000000456172732900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003f0000000c0000001600000006000000546174746f6f2900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000420000000c00000019000000090000004163636573736f72792900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000410000000c000000180000000800000048616e6468656c642900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000400000000c00000017000000070000005370656369616c2900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000".to_string(),
                dna_bytes: 16,
            },
        };
        (nervape_content, nervape_metadata)
    }

    fn generate_unicorn_dob_ingredients(
        onchain_decoder: bool,
    ) -> (SporeContentField, ClusterDescriptionField) {
        let unicorn_content = SporeContentField {
            id: None,
            block_number: Some(120),
            cell_id: Some(11844),
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
                hash: h256!("0xb82abd59ade361a014f0abb692f71b0feb880693c3ccb95b9137b73551d872ce"),
            }
        };
        let unicorn_metadata = ClusterDescriptionField {
            description: "Unicorn Collection".to_string(),
            dob: DOBClusterFormat {
                ver: None,
                decoder,
                pattern: "bc06000028000000d80000003b010000f9010000ed020000ec030000de0400003006000077060000b00000000c0000001700000007000000537069726974739900000008000000910000000c0000000d000000010000000080000000180000002b0000003d000000550000006b0000000f000000576f6f642c20426c756520426f64790e000000466972652c2052656420426f64791400000045617274682c20436f6c6f7266756c20426f6479120000004d6574616c2c20476f6c64656e20426f64791100000057617465722c20576869746520426f6479630000000c000000180000000800000059696e2059616e674b00000008000000430000000c0000000d0000000100000000320000000c000000200000001000000059616e672c2053686f727420486169720e00000059696e2c204c6f6e672068616972be0000000c000000170000000700000054616c656e7473a7000000080000009f0000000c0000000d00000001000000008e0000002c000000370000004000000049000000540000005d00000067000000710000007a00000084000000070000005265766976616c0500000044656174680500000043757273650700000050726f706865740500000043726f776e060000004865726d69740600000041747461636b0500000047756172640600000053756d6d6f6e06000000466f72676574f40000000c0000001400000004000000486f726ee000000008000000d80000000c0000000d0000000100000000c70000002c0000003b000000470000005b0000006a000000790000008900000099000000ac000000b90000000b0000005368616d616e20486f726e0800000048656c20486f726e100000004e6563726f6d616e63657220486f726e0b000000536962796c20486f726e200b00000043616573617220486f726e0c0000004c616f2054737520486f726e0c00000057617272696f7220486f726e0f00000050726165746f7269616e20486f726e090000004261726420486f726e0a0000004c6574686520486f726eff0000000c000000150000000500000057696e6773ea00000008000000e20000000c0000000d0000000100000000d10000002c0000003a000000500000006300000070000000800000008f000000a5000000b3000000c30000000a00000057696e642057696e6773120000004e6967687420536861646f772057696e67730f0000004c696768746e696e672057696e67730900000053756e2057696e67730c000000476f6c64656e2057696e67730b000000436c6f75642057696e6773120000004d6f726e696e6720476c6f772057696e67730a000000537461722057696e67730c000000537072696e672057696e67730a0000004d6f6f6e2057696e6773f20000000c00000015000000050000005461696c73dd00000008000000d50000000c0000000d0000000100000000c4000000280000003800000049000000590000006a0000008100000093000000a7000000b60000000c0000004d6574656f72205461696c730d0000005261696e626f77205461696c730c00000057696c6c6f77205461696c730d00000050686f656e6978205461696c731300000053756e73657420536861646f77205461696c730e000000536f637261746573205461696c731000000044756d626c65646f7265205461696c730b00000056656e7573205461696c730a00000047616961205461696c73520100000c0000001a0000000a000000486f72736573686f65733801000008000000300100000c0000000d00000001000000001f0100003000000042000000570000006a0000007e00000094000000a8000000bd000000d2000000ea000000020100000e00000049636520486f72736573686f65731100000044696d6f6e6420486f72736573686f65730f000000526f636b20486f72736573686f657310000000466c616d6520486f72736573686f6573120000005468756e64657220486f72736573686f6573100000004c6f74757320486f72736573686f65731100000053696c76657220486f72736573686f657311000000476f6c64656e20486f72736573686f657314000000526564204d61706c6520486f72736573686f657314000000426c7565204c616b6520486f72736573686f657319000000436f6c6f7266756c2053746f6e6520486f72736573686f6573470000000c0000001e0000000e00000044657374696e79204e756d6265722900000008000000210000000c0000000d000000040300000050c3000000000000a086010000000000450000000c0000001c0000000c0000004c75636b79204e756d6265722900000008000000210000000c0000000d000000010300000001000000000000003100000000000000".to_string(),
                dna_bytes: 12,
            },
        };
        (unicorn_content, unicorn_metadata)
    }

    fn decode_unicorn_dna(onchain_decoder: bool) -> String {
        let settings = prepare_settings("text/plain");
        let decoder = DOBDecoder::new(settings);
        let (unicorn_content, unicorn_metadata) = generate_unicorn_dob_ingredients(onchain_decoder);
        decoder
            .decode_dna(&unicorn_content, unicorn_metadata)
            .expect("decode")
    }

    #[test]
    fn test_decode_unicorn_dna() {
        let render_result = decode_unicorn_dna(false);
        assert_eq!(render_result, EXPECTED_UNICORN_RENDER_RESULT);
    }

    #[test]
    fn test_decode_unicorn_dna_with_onchain_decoder() {
        let render_result = decode_unicorn_dna(true);
        assert_eq!(render_result, EXPECTED_UNICORN_RENDER_RESULT);
    }

    #[test]
    fn test_fetch_and_decode_nervape_dna() {
        let settings = prepare_settings("text/plain");
        let decoder = DOBDecoder::new(settings);
        let (dob_content, dob_metadata) = decoder
            .fetch_decode_ingredients(NERVAPE_SPORE_ID.into())
            .expect("fetch");
        let render_result = decoder
            .decode_dna(&dob_content, dob_metadata)
            .expect("decode");
        assert_eq!(render_result, EXPECTED_NERVAPE_RENDER_RESULT);
    }

    #[test]
    #[should_panic = "fetch: DOBVersionUnexpected"]
    fn test_fetch_onchain_dob_failed() {
        let settings = prepare_settings("dob/0");
        DOBDecoder::new(settings)
            .fetch_decode_ingredients(UNICORN_SPORE_ID.into())
            .expect("fetch");
    }

    #[test]
    fn test_decode_onchain_nervape_dna_in_thread() {
        let protocol_version = "text/plain";
        let settings = prepare_settings(protocol_version);
        let (decoder, cmd) = DOBThreadDecoder::new(settings);
        decoder.run();
        assert_eq!(cmd.protocol_version(), protocol_version);
        let (render_result, _) = cmd
            .decode_dna("9dd9604d44d6640d1533c9f97f89438f17526e645f6c35aa08d8c7d844578580")
            .expect("thread decode");
        assert_eq!(render_result, EXPECTED_NERVAPE_RENDER_RESULT);
        cmd.stop();
    }

    #[test]
    fn test_nervape_json_serde() {
        let (nervape_content, nervape_metadata) = generate_nervape_dob_ingredients(false);
        let json_unicorn_content = serde_json::to_string(&nervape_content).unwrap();
        let json_unicorn_metadata = serde_json::to_string(&nervape_metadata).unwrap();
        println!("[spore_content] = {json_unicorn_content}");
        println!("[cluster_description] = {json_unicorn_metadata}");
        let deser_unicorn_content: SporeContentField =
            serde_json::from_slice(json_unicorn_content.as_bytes()).unwrap();
        let deser_unicorn_metadata: ClusterDescriptionField =
            serde_json::from_slice(json_unicorn_metadata.as_bytes()).unwrap();
        assert_eq!(nervape_content, deser_unicorn_content);
        assert_eq!(nervape_metadata, deser_unicorn_metadata);
    }

    #[test]
    fn test_unicorn_json_serde() {
        let (unicorn_content, unicorn_metadata) = generate_unicorn_dob_ingredients(true);
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
