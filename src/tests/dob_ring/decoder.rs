use ckb_types::{h256, packed::OutPoint, prelude::Entity};
use serde_json::{json, Value};

use crate::{
    decoder::DOBDecoder,
    tests::{prepare_settings, dob_ring::client::MockRpcClient},
    types::{
        ClusterDescriptionField, DOBClusterFormat, DOBClusterFormatV0, DOBClusterFormatV1,
        DOBDecoderFormat, DecoderLocationType,
    },
};

fn generate_dob_ring_ingredients() -> (Value, ClusterDescriptionField) {
    let content = json!({
        "dna": hex::encode(OutPoint::new(Default::default(), 0).as_bytes())
    });
    let metadata = ClusterDescriptionField {
        description: "Ring DOB Test".to_string(),
        dob: DOBClusterFormat::new_dob1(DOBClusterFormatV1 {
            decoders: vec![
                DOBClusterFormatV0 {
                    decoder: DOBDecoderFormat {
                        location: DecoderLocationType::CodeHash,
                        hash: Some(h256!(
                            "0x198c5ccb3fbd3309f110b8bdbc3df086bc9fb3867716f4e203005c501b172f00"
                        )),
                        script: None
                    },
                    pattern: serde_json::from_str("[[\"Name\",\"String\",\"0000000000000000000000000000000000000000000000000000000000000000\",0,1,\"options\",[\"Alice\",\"Bob\",\"Charlie\",\"David\",\"Ethan\",\"Florence\",\"Grace\",\"Helen\"]],[\"Age\",\"Number\",\"0101010101010101010101010101010101010101010101010101010101010101\",1,1,\"range\",[0,100]],[\"Score\",\"Number\",\"0202020202020202020202020202020202020202020202020202020202020202\",2,1,\"rawNumber\"],[\"DNA\",\"String\",\"0303030303030303030303030303030303030303030303030303030303030303\",3,3,\"rawString\"],[\"URL\",\"String\",\"0404040404040404040404040404040404040404040404040404040404040404\",6,30,\"utf8\"],[\"Value\",\"Timestamp\",\"0505050505050505050505050505050505050505050505050505050505050505\",3,3,\"rawNumber\"]]").unwrap(),
                }, 
                DOBClusterFormatV0 {
                    decoder: DOBDecoderFormat {
                        location: DecoderLocationType::TypeScript,
                        script: Some(serde_json::from_str(r#"
                        {
                            "code_hash": "0x00000000000000000000000000000000000000000000000000545950455f4944",
                            "hash_type": "type",
                            "args": "0x784e32cef202b9d4759ea96e80d806f94051e8069fd34d761f452553700138d7"
                        }
                        "#).unwrap()
                        ),
                        hash: None,
                    },
                    pattern: serde_json::from_str("[[\"IMAGE.0\",\"attributes\",\"\",\"raw\",\"width='200' height='200' xmlns='http://www.w3.org/2000/svg'\"],[\"IMAGE.0\",\"elements\",\"Name\",\"options\",[[\"Alice\",\"<rect fill='#0000FF' width='200' height='200' />\"],[\"Bob\",\"<rect fill='#00FF00' width='200' height='200' />\"],[\"Ethan\",\"<rect fill='#FF0000' width='200' height='200' />\"],[[\"*\"],\"<rect fill='#FFFFFF' width='200' height='200' />\"]]],[\"IMAGE.0\",\"elements\",\"Age\",\"range\",[[[0,50],\"<image width='200' height='200' href='btcfs://b2f4560f17679d3e3fca66209ac425c660d28a252ef72444c3325c6eb0364393i0' />\"],[[51,100],\"<image width='200' height='200' href='btcfs://eb3910b3e32a5ed9460bd0d75168c01ba1b8f00cc0faf83e4d8b67b48ea79676i0' />\"],[[\"*\"],\"<image width='200' height='200' href='btcfs://11b6303eb7d887d7ade459ac27959754cd55f9f9e50345ced8e1e8f47f4581fai0' />\"]]],[\"IMAGE.1\",\"attributes\",\"\",\"raw\",\"xmlns='http://www.w3.org/2000/svg'\"],[\"IMAGE.1\",\"elements\",\"Score\",\"range\",[[[0,1000],\"<image width='200' height='200' href='ipfs://QmeQ6TfqzsjJCMtYmpbyZeMxiSzQGc6Aqg6NyJTeLYrrJr' />\"],[[\"*\"],\"<image width='200' height='200' href='ipfs://QmWjv41cCvGn6sf1zh8pAokX1Nf5oShz8EtLaxxKQLyJfW' />\"]]]]").unwrap(),
                }
            ],
        }),
    };
    (content, metadata)
}

#[test]
fn test_print_dob_ring_ingreidents() {
    let (_, dob_metadata) = generate_dob_ring_ingredients();
    println!(
        "cluster_description: {}",
        serde_json::to_string(&dob_metadata).unwrap()
    );
}

#[tokio::test]
async fn test_dob_ring_decode() {
    let settings = prepare_settings("dob/1");
    let (content, dob_metadata) = generate_dob_ring_ingredients();
    let rpc = MockRpcClient::new(&settings.ckb_rpc, None);
    let decoder = DOBDecoder::new(rpc, settings);
    let dna = content.get("dna").unwrap().as_str().unwrap();
    let render_result = decoder
        .decode_dna(dna, dob_metadata, Default::default())
        .await
        .expect("decode");
    println!("\nrender_result: {}", render_result);
}
