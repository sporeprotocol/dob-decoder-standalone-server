mod vm;

pub mod client;
pub mod decoder;
pub mod types;

#[cfg(test)]
mod test {
    use ckb_types::{h256, H256};

    use crate::decoder::{decode_spore_data, DOBDecoder};
    use crate::types::{
        ClusterDescriptionField, DOBClusterFormat, DOBDecoderFormat, DecoderLocationType, Settings,
        SporeContentFieldObject,
    };

    const EXPECTED_UNICORN_RENDER_RESULT: &str = "[{\"name\":\"wuxing_yinyang\",\"traits\":[{\"String\":\"3<_>\"}]},{\"name\":\"prev.bgcolor\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['#DBAB00', '#09D3FF', '#A028E9', '#FF3939', '#(135deg, #FE4F4F, #66C084, #00E2E2, #E180E2, #F4EC32)']\"}]},{\"name\":\"prev<%v>\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['#000000', '#000000', '#000000', '#000000', '#000000', '#FFFFFF', '#FFFFFF', '#FFFFFF', '#FFFFFF', '#FFFFFF'])\"}]},{\"name\":\"Spirits\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['Metal, Golden Body', 'Wood, Blue Body', 'Water, White Body', 'Fire, Red Body', 'Earth, Colorful Body']\"}]},{\"name\":\"Yin Yang\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['Yin, Long hair', 'Yin, Long hair', 'Yin, Long hair', 'Yin, Long hair', 'Yin, Long hair', 'Yang, Short Hair', 'Yang, Short Hair', 'Yang, Short Hair', 'Yang, Short Hair', 'Yang, Short Hair']\"}]},{\"name\":\"Talents\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['Guard<~>', 'Death<~>', 'Forget<~>', 'Curse<~>', 'Hermit<~>', 'Attack<~>', 'Revival<~>', 'Summon<~>', 'Prophet<~>', 'Crown<~>']\"}]},{\"name\":\"Horn\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['Praetorian Horn', 'Hel Horn', 'Lethe Horn', 'Necromancer Horn', 'Lao Tsu Horn', 'Warrior Horn', 'Shaman Horn', 'Bard Horn', 'Sibyl Horn', 'Caesar Horn']\"}]},{\"name\":\"Wings\",\"traits\":[{\"String\":\"Sun Wings\"}]},{\"name\":\"Tails\",\"traits\":[{\"String\":\"Meteor Tail\"}]},{\"name\":\"Horseshoes\",\"traits\":[{\"String\":\"Silver Horseshoes\"}]},{\"name\":\"Destiny Number\",\"traits\":[{\"Number\":59616}]},{\"name\":\"Lucky Number\",\"traits\":[{\"Number\":35}]}]";
    const EXPECTED_NERVAPE_RENDER_RESULT: &str = "[{\"name\":\"prev.type\",\"traits\":[{\"String\":\"text\"}]},{\"name\":\"prev.bg\",\"traits\":[{\"String\":\"btcfs://59e87ca177ef0fd457e87e9f93627660022cf519b531e1f4e3a6dda9e5e33827i0\"}]},{\"name\":\"prev.bgcolor\",\"traits\":[{\"String\":\"#CEBAF7\"}]},{\"name\":\"Background\",\"traits\":[{\"Number\":170}]},{\"name\":\"Suit\",\"traits\":[{\"Number\":236}]},{\"name\":\"Upper body\",\"traits\":[{\"Number\":53}]},{\"name\":\"Lower body\",\"traits\":[{\"Number\":189}]},{\"name\":\"Headwear\",\"traits\":[{\"Number\":175}]},{\"name\":\"Mask\",\"traits\":[{\"Number\":153}]},{\"name\":\"Eyewear\",\"traits\":[{\"Number\":126}]},{\"name\":\"Mouth\",\"traits\":[{\"Number\":14}]},{\"name\":\"Ears\",\"traits\":[{\"Number\":165}]},{\"name\":\"Tattoo\",\"traits\":[{\"Number\":231}]},{\"name\":\"Accessory\",\"traits\":[{\"Number\":78}]},{\"name\":\"Handheld\",\"traits\":[{\"Number\":240}]},{\"name\":\"Special\",\"traits\":[{\"Number\":70}]}]";
    const NERVAPE_SPORE_ID: H256 =
        h256!("0x9dd9604d44d6640d1533c9f97f89438f17526e645f6c35aa08d8c7d844578580");

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
            ..Default::default()
        }
    }

    fn generate_nervape_dob_ingredients(
        onchain_decoder: bool,
    ) -> (SporeContentFieldObject, ClusterDescriptionField) {
        let nervape_content = SporeContentFieldObject {
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
                pattern: "830900004400000087000000370500004206000085060000c2060000050700004807000089070000c6070000060800004408000081080000c00800000209000043090000430000000c0000001900000009000000707265762e747970652a00000008000000220000000c0000000d0000000100000000110000000800000005000000696d616765b00400000c0000001700000007000000707265762e62679904000008000000910400000c0000000d0000000100000000800400003c0000008a000000d80000002601000074010000c2010000100200005e020000ac020000fa0200004803000096030000e4030000320400004a00000062746366733a2f2f3162633234333531613064663265363836353734636431623633343661316635356638316366663061326535323037386136653361643061333563666238333369304a00000062746366733a2f2f3634663536326431366532613461323965386334383231333730666666343733656466613232633236656635383038616462323430346533396463303133653569304a00000062746366733a2f2f6332396665636436643764376565633063623361326233646664636236616132363038316462386639383531313130623763323061306633633631373239396169304a00000062746366733a2f2f3539653837636131373765663066643435376538376539663933363237363630303232636635313962353331653166346533613664646139653565333338323769304a00000062746366733a2f2f6133353839646463663462376133633664613532666536616534656433323936663165646531333966653931323766323639376365306463663237303362363169304a00000062746366733a2f2f3739393732396666366131366464366166353764623161386361363134366435363733613330616439613539373664643836316433343861356565633238633469304a00000062746366733a2f2f3838646432616230356262386639633732646134326166633730363737616330356634373665313765306631363535316463303036333561653765393534366569304a00000062746366733a2f2f6233326533626262373363623837376339623431313532393933306135623665623332383039323762323832633132343836636532363930316233633232393169304a00000062746366733a2f2f6138623139646461623333386462306335326639613238346237643935666665616130646533346530623837343137373930316562393265306639663964386469304a00000062746366733a2f2f6261386231626239643862616565346266323461303666616132356235363934313066326462393662343633396638653038636362656330356338386437396269304a00000062746366733a2f2f6161383938366630656636363738303764346232333937306536343834346464653366303632323534326237396135633330323533396465306333356233316569304a00000062746366733a2f2f3130306637653066303936356463353435313561333833316133323038383133313563663563613634616430316265643262343232363136623135666433313469304a00000062746366733a2f2f6238346563306337373061613139363161336439343938656138613637653132383235333239313366633163313365336561663561343864653231363466623969304a00000062746366733a2f2f6130366261326531363134613530393931373665356363346439356465373663626562343730356138626437653134323333363237386562633239306664623369300b0100000c0000001c0000000c000000707265762e6267636f6c6f72ef00000008000000e70000000c0000000d0000000100000000d60000003c00000047000000520000005d00000068000000730000007e00000089000000940000009f000000aa000000b5000000c0000000cb00000007000000234646453345420700000023464643324645070000002343454241463707000000234237453646390700000023414246344430070000002345304446424407000000234639463741370700000023453242453931070000002346394336363207000000234637443642320700000023464341383633070000002346394143414307000000234530453145320700000023413341374141430000000c0000001a0000000a0000004261636b67726f756e642900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003d0000000c0000001400000004000000537569742900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000430000000c0000001a0000000a000000557070657220626f64792900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000430000000c0000001a0000000a0000004c6f77657220626f64792900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000410000000c000000180000000800000048656164776561722900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003d0000000c00000014000000040000004d61736b2900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000400000000c0000001700000007000000457965776561722900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003e0000000c00000015000000050000004d6f7574682900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003d0000000c0000001400000004000000456172732900000008000000210000000c0000000d00000001030000000000000000000000ff000000000000003f0000000c0000001600000006000000546174746f6f2900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000420000000c00000019000000090000004163636573736f72792900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000410000000c000000180000000800000048616e6468656c642900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000400000000c00000017000000070000005370656369616c2900000008000000210000000c0000000d00000001030000000000000000000000ff00000000000000".to_string(),
                dna_bytes: 16,
            },
        };
        (nervape_content, nervape_metadata)
    }

    fn generate_unicorn_dob_ingredients(
        onchain_decoder: bool,
    ) -> (SporeContentFieldObject, ClusterDescriptionField) {
        let unicorn_content = SporeContentFieldObject {
            id: None,
            block_number: Some(120),
            cell_id: Some(11844),
            dna: "df4ffcb5e7a283ea7e6f09a504d0e256".to_string(),
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
            description: "Unicorns are the first series of digital objects generated based on time and space on CKB. Combining the Birth Time-location Determining Destiny Theory, Five Element Theory and YinYang Theory, it provide a special way for people to get Unicorn's on-chain DNA. Now all the seeds(DNAs) are on chain, and a magic world can expand.".to_string(),
            dob: DOBClusterFormat {
                ver: Some(0),
                decoder,
                pattern: "3d09000034000000e7000000a00100005e0200001403000021040000ef040000d4050000e6060000cf070000b1080000f8080000b30000000c0000001e0000000e000000777578696e675f79696e79616e6795000000080000008d0000000c0000000d00000001000000007c0000002c000000340000003c000000440000004c000000540000005c000000640000006c0000007400000004000000303c5f3e04000000313c5f3e04000000323c5f3e04000000333c5f3e04000000343c5f3e04000000353c5f3e04000000363c5f3e04000000373c5f3e04000000383c5f3e04000000393c5f3eb90000000c0000001c0000000c000000707265762e6267636f6c6f729d00000008000000950000000c0000000d00000001000000008400000008000000780000002825777578696e675f79696e79616e67293a5b2723444241423030272c202723303944334646272c202723413032384539272c202723464633393339272c202723283133356465672c20234645344634462c20233636433038342c20233030453245322c20234531383045322c202346344543333229275dbe0000000c0000001800000008000000707265763c25763ea6000000080000009e0000000c0000000d00000001000000008d00000008000000810000002825777578696e675f79696e79616e67293a5b2723303030303030272c202723303030303030272c202723303030303030272c202723303030303030272c202723303030303030272c202723464646464646272c202723464646464646272c202723464646464646272c202723464646464646272c202723464646464646275d29b60000000c0000001700000007000000537069726974739f00000008000000970000000c0000000d000000010000000086000000080000007a0000002825777578696e675f79696e79616e67293a5b274d6574616c2c20476f6c64656e20426f6479272c2027576f6f642c20426c756520426f6479272c202757617465722c20576869746520426f6479272c2027466972652c2052656420426f6479272c202745617274682c20436f6c6f7266756c20426f6479275d0d0100000c000000180000000800000059696e2059616e67f500000008000000ed0000000c0000000d0000000100000000dc00000008000000d00000002825777578696e675f79696e79616e67293a5b2759696e2c204c6f6e672068616972272c202759696e2c204c6f6e672068616972272c202759696e2c204c6f6e672068616972272c202759696e2c204c6f6e672068616972272c202759696e2c204c6f6e672068616972272c202759616e672c2053686f72742048616972272c202759616e672c2053686f72742048616972272c202759616e672c2053686f72742048616972272c202759616e672c2053686f72742048616972272c202759616e672c2053686f72742048616972275dce0000000c000000170000000700000054616c656e7473b700000008000000af0000000c0000000d00000001000000009e00000008000000920000002825777578696e675f79696e79616e67293a5b2747756172643c7e3e272c202744656174683c7e3e272c2027466f726765743c7e3e272c202743757273653c7e3e272c20274865726d69743c7e3e272c202741747461636b3c7e3e272c20275265766976616c3c7e3e272c202753756d6d6f6e3c7e3e272c202750726f706865743c7e3e272c202743726f776e3c7e3e275de50000000c0000001400000004000000486f726ed100000008000000c90000000c0000000d0000000100000000b800000008000000ac0000002825777578696e675f79696e79616e67293a5b2750726165746f7269616e20486f726e272c202748656c20486f726e272c20274c6574686520486f726e272c20274e6563726f6d616e63657220486f726e272c20274c616f2054737520486f726e272c202757617272696f7220486f726e272c20275368616d616e20486f726e272c20274261726420486f726e272c2027536962796c20486f726e272c202743616573617220486f726e275d120100000c000000150000000500000057696e6773fd00000008000000f50000000c0000000d0000000100000000e4000000300000003e0000005400000067000000740000008400000093000000a9000000b7000000c7000000d50000000a00000057696e642057696e6773120000004e6967687420536861646f772057696e67730f0000004c696768746e696e672057696e67730900000053756e2057696e67730c000000476f6c64656e2057696e67730b000000436c6f75642057696e6773120000004d6f726e696e6720476c6f772057696e67730a000000537461722057696e67730c000000537072696e672057696e67730a0000004d6f6f6e2057696e67730b000000416e67656c2057696e6773e90000000c00000015000000050000005461696c73d400000008000000cc0000000c0000000d0000000100000000bb00000028000000370000004700000056000000660000007c0000008d000000a0000000ae0000000b0000004d6574656f72205461696c0c0000005261696e626f77205461696c0b00000057696c6c6f77205461696c0c00000050686f656e6978205461696c1200000053756e73657420536861646f77205461696c0d000000536f637261746573205461696c0f00000044756d626c65646f7265205461696c0a00000056656e7573205461696c0900000047616961205461696ce20000000c0000001a0000000a000000486f72736573686f6573c800000008000000c00000000c0000000d0000000100000000af0000002000000032000000480000005c00000070000000860000009a0000000e00000049636520486f72736573686f6573120000004372797374616c20486f72736573686f6573100000004d61706c6520486f72736573686f657310000000466c616d6520486f72736573686f6573120000005468756e64657220486f72736573686f6573100000004c6f74757320486f72736573686f65731100000053696c76657220486f72736573686f6573470000000c0000001e0000000e00000044657374696e79204e756d6265722900000008000000210000000c0000000d000000040300000050c3000000000000a086010000000000450000000c0000001c0000000c0000004c75636b79204e756d6265722900000008000000210000000c0000000d000000010300000001000000000000003100000000000000".to_string(),
                dna_bytes: 16,
            },
        };
        (unicorn_content, unicorn_metadata)
    }

    async fn decode_unicorn_dna(onchain_decoder: bool) -> String {
        let settings = prepare_settings("text/plain");
        let decoder = DOBDecoder::new(settings);
        let (unicorn_content, unicorn_metadata) = generate_unicorn_dob_ingredients(onchain_decoder);
        decoder
            .decode_dna(&[&unicorn_content.dna], unicorn_metadata)
            .await
            .expect("decode")
    }

    #[tokio::test]
    async fn test_decode_unicorn_dna() {
        let render_result = decode_unicorn_dna(false).await;
        assert_eq!(render_result, EXPECTED_UNICORN_RENDER_RESULT);
    }

    #[tokio::test]
    async fn test_decode_unicorn_dna_with_onchain_decoder() {
        let render_result = decode_unicorn_dna(true).await;
        assert_eq!(render_result, EXPECTED_UNICORN_RENDER_RESULT);
    }

    #[tokio::test]
    async fn test_fetch_and_decode_nervape_dna() {
        let settings = prepare_settings("text/plain");
        let decoder = DOBDecoder::new(settings);
        let (dob_content, dob_metadata) = decoder
            .fetch_decode_ingredients(NERVAPE_SPORE_ID.into())
            .await
            .expect("fetch");
        let render_result = decoder
            .decode_dna(&dob_content.dna_set(), dob_metadata)
            // array type
            .await
            .expect("decode");
        assert_eq!(render_result, EXPECTED_NERVAPE_RENDER_RESULT);
    }

    #[tokio::test]
    #[should_panic = "fetch: DOBVersionUnexpected"]
    async fn test_fetch_onchain_dob_failed() {
        let settings = prepare_settings("dob/0");
        DOBDecoder::new(settings)
            .fetch_decode_ingredients(NERVAPE_SPORE_ID.into())
            .await
            .expect("fetch");
    }

    #[test]
    fn test_nervape_json_serde() {
        let (nervape_content, nervape_metadata) = generate_nervape_dob_ingredients(false);
        let json_unicorn_content = serde_json::to_string(&nervape_content).unwrap();
        let json_unicorn_metadata = serde_json::to_string(&nervape_metadata).unwrap();
        println!("[spore_content] = {json_unicorn_content}");
        println!("[cluster_description] = {json_unicorn_metadata}");
        let deser_unicorn_content: SporeContentFieldObject =
            serde_json::from_slice(json_unicorn_content.as_bytes()).unwrap();
        let deser_unicorn_metadata: ClusterDescriptionField =
            serde_json::from_slice(json_unicorn_metadata.as_bytes()).unwrap();
        assert_eq!(nervape_content, deser_unicorn_content);
        assert_eq!(nervape_metadata, deser_unicorn_metadata);
    }

    #[test]
    fn test_unicorn_json_serde() {
        let (unicorn_content, unicorn_metadata) = generate_unicorn_dob_ingredients(false);
        let json_unicorn_content = serde_json::to_string(&unicorn_content).unwrap();
        let json_unicorn_metadata = serde_json::to_string(&unicorn_metadata).unwrap();
        println!("[spore_content] = {json_unicorn_content}");
        println!("[cluster_description] = {json_unicorn_metadata}");
        let deser_unicorn_content: SporeContentFieldObject =
            serde_json::from_slice(json_unicorn_content.as_bytes()).unwrap();
        let deser_unicorn_metadata: ClusterDescriptionField =
            serde_json::from_slice(json_unicorn_metadata.as_bytes()).unwrap();
        assert_eq!(unicorn_content, deser_unicorn_content);
        assert_eq!(unicorn_metadata, deser_unicorn_metadata);
    }

    #[test]
    fn test_decode_multiple_spore_data() {
        let dna = "eda7a47a751d2dc42d4b724e47cfd67a";
        [
            format!("{{\"dna\": \"{dna}\"}}"), // object type
            format!("[\"{dna}\"]"),            // array type
            format!("\"{dna}\""),              // string type
        ]
        .into_iter()
        .enumerate()
        .for_each(|(i, spore_data)| {
            let dob_content =
                decode_spore_data(spore_data.as_bytes()).expect(&format!("assert type index {i}"));
            dob_content.dna_set().into_iter().for_each(|v| {
                assert_eq!(v, dna, "object type comparison failed");
            });
        });
    }
}
