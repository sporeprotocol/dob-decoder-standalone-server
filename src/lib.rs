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
                pattern: "c70500000c000000bc040000b00400000c0000001700000007000000707265762e62679904000008000000910400000c0000000d0000000100000000800400003c0000008a000000d80000002601000074010000c2010000100200005e020000ac020000fa0200004803000096030000e4030000320400004a00000062746366733a2f2f3162633234333531613064663265363836353734636431623633343661316635356638316366663061326535323037386136653361643061333563666238333369304a00000062746366733a2f2f3634663536326431366532613461323965386334383231333730666666343733656466613232633236656635383038616462323430346533396463303133653569304a00000062746366733a2f2f6332396665636436643764376565633063623361326233646664636236616132363038316462386639383531313130623763323061306633633631373239396169304a00000062746366733a2f2f3539653837636131373765663066643435376538376539663933363237363630303232636635313962353331653166346533613664646139653565333338323769304a00000062746366733a2f2f6133353839646463663462376133633664613532666536616534656433323936663165646531333966653931323766323639376365306463663237303362363169304a00000062746366733a2f2f3739393732396666366131366464366166353764623161386361363134366435363733613330616439613539373664643836316433343861356565633238633469304a00000062746366733a2f2f3838646432616230356262386639633732646134326166633730363737616330356634373665313765306631363535316463303036333561653765393534366569304a00000062746366733a2f2f6233326533626262373363623837376339623431313532393933306135623665623332383039323762323832633132343836636532363930316233633232393169304a00000062746366733a2f2f6138623139646461623333386462306335326639613238346237643935666665616130646533346530623837343137373930316562393265306639663964386469304a00000062746366733a2f2f6261386231626239643862616565346266323461303666616132356235363934313066326462393662343633396638653038636362656330356338386437396269304a00000062746366733a2f2f6161383938366630656636363738303764346232333937306536343834346464653366303632323534326237396135633330323533396465306333356233316569304a00000062746366733a2f2f3130306637653066303936356463353435313561333833316133323038383133313563663563613634616430316265643262343232363136623135666433313469304a00000062746366733a2f2f6238346563306337373061613139363161336439343938656138613637653132383235333239313366633163313365336561663561343864653231363466623969304a00000062746366733a2f2f6130366261326531363134613530393931373665356363346439356465373663626562343730356138626437653134323333363237386562633239306664623369300b0100000c0000001c0000000c000000707265762e6267636f6c6f72ef00000008000000e70000000c0000000d0000000100000000d60000003c00000047000000520000005d00000068000000730000007e00000089000000940000009f000000aa000000b5000000c0000000cb00000007000000234646453345420700000023464643324645070000002343454241463707000000234237453646390700000023414246344430070000002345304446424407000000234639463741370700000023453242453931070000002346394336363207000000234637443642320700000023464341383633070000002346394143414307000000234530453145320700000023413341374141".to_string(),
                dna_bytes: 8,
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
        let (nervape_content, nervape_metadata) = generate_dob_ingredients(true);
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
}
