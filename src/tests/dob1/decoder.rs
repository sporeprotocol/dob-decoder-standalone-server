use ckb_types::h256;
use serde_json::{json, Value};

use crate::{
    decoder::DOBDecoder,
    tests::prepare_settings,
    types::{
        ClusterDescriptionField, DOBClusterFormat, DOBClusterFormatV0, DOBClusterFormatV1,
        DOBDecoderFormat, DecoderLocationType,
    },
};

fn generate_dob1_ingredients() -> (Value, ClusterDescriptionField) {
    let content = json!({
        "dna": "ac7b88aabbcc687474703a2f2f3132372e302e302e313a383039300000"
    });
    let metadata = ClusterDescriptionField {
        description: "DOB/1 Test".to_string(),
        dob: DOBClusterFormat::new_dob1(DOBClusterFormatV1 {
            decoders: vec![
                DOBClusterFormatV0 {
                    decoder: DOBDecoderFormat {
                        location: DecoderLocationType::CodeHash,
                        hash: Some(h256!(
                            "0x13cac78ad8482202f18f9df4ea707611c35f994375fa03ae79121312dda9925c"
                        )),
                        script: None
                    },
                    pattern: serde_json::from_str("[[\"Name\",\"String\",0,1,\"options\",[\"Alice\",\"Bob\",\"Charlie\",\"David\",\"Ethan\",\"Florence\",\"Grace\",\"Helen\"]],[\"Age\",\"Number\",1,1,\"range\",[0,100]],[\"Score\",\"Number\",2,1,\"rawNumber\"],[\"_DNA\",\"String\",3,3,\"rawString\"],[\"_URL\",\"string\",6,21,\"utf8\"],[\"Value\",\"Number\",3,3,\"rawNumber\"]]").unwrap(),
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
                    pattern: serde_json::from_str("[[\"IMAGE.0\",\"attributes\",\"\",\"raw\",\"width='200' height='200' xmlns='http://www.w3.org/2000/svg'\"],[\"IMAGE.0\",\"attributes\",\"Name\",\"options\",[[\"Alice\",\"fill='#0000FF'\"],[\"Bob\",\"fill='#00FF00'\"],[\"Ethan\",\"fill='#FF0000'\"],[[\"*\"],\"fill='#FFFFFF'\"]]],[\"IMAGE.0\",\"elements\",\"Age\",\"range\",[[[0,50],\"<image href='btcfs://b2f4560f17679d3e3fca66209ac425c660d28a252ef72444c3325c6eb0364393i0' />\"],[[51,100],\"<image href='btcfs://eb3910b3e32a5ed9460bd0d75168c01ba1b8f00cc0faf83e4d8b67b48ea79676i0' />\"],[[\"*\"],\"<image href='btcfs://11b6303eb7d887d7ade459ac27959754cd55f9f9e50345ced8e1e8f47f4581fai0' />\"]]]]").unwrap(),
                }
            ],
        }),
    };
    (content, metadata)
}

#[test]
fn test_print_dob1_ingreidents() {
    let (_, dob_metadata) = generate_dob1_ingredients();
    println!(
        "cluster_description: {}",
        serde_json::to_string(&dob_metadata).unwrap()
    );
}

#[tokio::test]
async fn test_dob1_basic_decode() {
    let settings = prepare_settings("dob/1");
    let (content, dob_metadata) = generate_dob1_ingredients();
    let decoder = DOBDecoder::new(settings);
    let dna = content.get("dna").unwrap().as_str().unwrap();
    let render_result = decoder.decode_dna(dna, dob_metadata).await.expect("decode");
    println!("\nrender_result: {}", render_result);
}
