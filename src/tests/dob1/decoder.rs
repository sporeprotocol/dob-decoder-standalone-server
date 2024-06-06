use ckb_types::h256;

use crate::client::ImageFetchClient;
use crate::decoder::DOBDecoder;
use crate::tests::prepare_settings;
use crate::types::{
    ClusterDescriptionField, DOBClusterFormat, DOBClusterFormatV0, DOBClusterFormatV1,
    DOBDecoderFormat, DecoderLocationType,
};
use serde_json::{json, Value};

fn generate_dob1_ingredients() -> (Value, ClusterDescriptionField) {
    let content = json!({
        "dna": "ac7b88aabbcc687474703a2f2f3132372e302e302e313a383039300000"
    });
    let metadata = ClusterDescriptionField {
        description: "DOB/1 Test".to_string(),
        dob: DOBClusterFormat::new_dob1(DOBClusterFormatV1 {
            traits: DOBClusterFormatV0 {
                decoder: DOBDecoderFormat {
                    location: DecoderLocationType::CodeHash,
                    hash: h256!(
                        "0x32f29aba4b17f3d05bec8cec55d50ef86766fd0bf82fdedaa14269f344d3784a"
                    ),
                },
                pattern: serde_json::from_str("[[\"Name\",\"string\",0,1,\"options\",[\"Alice\",\"Bob\",\"Charlie\",\"David\",\"Ethan\",\"Florence\",\"Grace\",\"Helen\"]],[\"Age\",\"number\",1,1,\"range\",[0,100]],[\"Score\",\"number\",2,1,\"raw\"],[\"DNA\",\"string\",3,3,\"raw\"],[\"URL\",\"string\",6,21,\"utf8\"],[\"Value\",\"number\",3,3,\"raw\"]]").unwrap(),
            },
            images: DOBClusterFormatV0 {
                decoder: DOBDecoderFormat {
                    location: DecoderLocationType::CodeHash,
                    hash: h256!(
                        "0x0000000000000000000000000000000000000000000000000000000000000000"
                    ),
                },
                pattern: Value::String("hello world".to_string()),
            },
        }),
    };
    (content, metadata)
}

#[tokio::test]
async fn check_fetched_image() {
    let mut fetcher = ImageFetchClient::new("https://mempool.space/api/tx/", 100);
    let uris = vec![
        "btcfs://b2f4560f17679d3e3fca66209ac425c660d28a252ef72444c3325c6eb0364393i0".to_string(),
    ];
    let images = fetcher.fetch_images(&uris).await.expect("fetch images");
    let image_raw_bytes = images.first().expect("image");
    image::load_from_memory(&image_raw_bytes).expect("load image");
}

#[tokio::test]
async fn test_dob1() {
    let settings = prepare_settings("text/plain");
    let decoder = DOBDecoder::new(settings);
    let (content, metadata) = generate_dob1_ingredients();
    decoder
        .decode_dna(&content["dna"].as_str().unwrap(), metadata)
        .await
        .expect("decode dob/1");
}
