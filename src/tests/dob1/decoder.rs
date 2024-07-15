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
                            "0x32f29aba4b17f3d05bec8cec55d50ef86766fd0bf82fdedaa14269f344d3784a"
                        )),
                        script: None
                    },
                    pattern: serde_json::from_str("[[\"Name\",\"string\",0,1,\"options\",[\"Alice\",\"Bob\",\"Charlie\",\"David\",\"Ethan\",\"Florence\",\"Grace\",\"Helen\"]],[\"Age\",\"number\",1,1,\"range\",[0,100]],[\"Score\",\"number\",2,1,\"raw\"],[\"DNA\",\"string\",3,3,\"raw\"],[\"URL\",\"string\",6,21,\"utf8\"],[\"Value\",\"number\",3,3,\"raw\"]]").unwrap(),
                },
                DOBClusterFormatV0 {
                    decoder: DOBDecoderFormat {
                        location: DecoderLocationType::CodeHash,
                        hash: Some(h256!(
                            "0x0d55a92f83e9af0addd335a34c762c93b55c4f4cd4eb136eb2a4c4c0d71a3ffa"
                        )),
                        script: None
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
