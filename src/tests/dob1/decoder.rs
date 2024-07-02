use std::collections::HashMap;
use std::io::Cursor;

use ckb_types::h256;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use serde_json::{json, Value};

use crate::client::ImageFetchClient;
use crate::decoder::DOBDecoder;
use crate::hashmap;
use crate::tests::prepare_settings;
use crate::types::{
    ClusterDescriptionField, DOBClusterFormat, DOBClusterFormatV0, DOBClusterFormatV1,
    DOBDecoderFormat, DecoderLocationType,
};

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
                        "0xac35b0e6178dc4a89fb85194b4ac0b60eed2b6ce9f10bf7bf2ee76190c3a0071"
                    ),
                },
                pattern: serde_json::from_str("[[\"0\",\"color\",\"Name\",\"options\",[[\"Alice\",\"#0000FF\"],[\"Bob\",\"#00FF00\"],[\"Ethan\",\"#FF0000\"],[[\"*\"],\"#FFFFFF\"]]],[\"0\",\"uri\",\"Age\",\"range\",[[[0,50],\"btcfs://b2f4560f17679d3e3fca66209ac425c660d28a252ef72444c3325c6eb0364393i0\"],[[51,100],\"btcfs://eb3910b3e32a5ed9460bd0d75168c01ba1b8f00cc0faf83e4d8b67b48ea79676i0\"],[[\"*\"],\"btcfs://11b6303eb7d887d7ade459ac27959754cd55f9f9e50345ced8e1e8f47f4581fai0\"]]],[\"0\",\"uri\",\"Score\",\"range\",[[[0,1000],\"btcfs://11d6cc654f4c0759bfee520966937a4304db2b33880c88c2a6c649e30c7b9aaei1\"],[[\"*\"],\"btcfs://e1484915b27e45b120239080fe5032580550ff9ff759eb26ee86bf8aaf90068bi1\"]]],[\"1\",\"uri\",\"Value\",\"range\",[[[0,100000],\"btcfs://11d6cc654f4c0759bfee520966937a4304db2b33880c88c2a6c649e30c7b9aaei0\"],[[\"*\"],\"btcfs://e1484915b27e45b120239080fe5032580550ff9ff759eb26ee86bf8aaf90068bi0\"]]]]").unwrap(),
            },
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

macro_rules! png_decoder {
    ($image:expr) => {{
        let rgba = image::load_from_memory(&$image).expect("load image");
        let mut inner_buffer = Vec::new();
        let buffer = Cursor::new(&mut inner_buffer);
        let encoder =
            PngEncoder::new_with_quality(buffer, CompressionType::Best, FilterType::NoFilter);
        rgba.write_with_encoder(encoder).expect("write image");
        inner_buffer
    }};
    () => {};
}

#[tokio::test]
async fn check_fetched_image() {
    let url = hashmap!(("btcfs", "https://mempool.space/api/tx/"));
    let mut fetcher = ImageFetchClient::new(&url, 100);
    let uris = vec![
        "btcfs://11d6cc654f4c0759bfee520966937a4304db2b33880c88c2a6c649e30c7b9aaei0".to_string(),
    ];
    let images = fetcher.fetch_images(&uris).await.expect("fetch images");
    let image_raw_bytes = images.first().expect("image");
    let png = png_decoder!(image_raw_bytes);
    println!("image size: {:?}", png.len());
    // std::fs::write("dob1.png", &png).expect("write image");
}

#[tokio::test]
async fn check_ipfs_image() {
    let url = hashmap!(("ipfs", "https://ipfs.io/ipfs/"));
    let mut fetcher = ImageFetchClient::new(&url, 100);
    let uris = vec!["ipfs://QmeQ6TfqzsjJCMtYmpbyZeMxiSzQGc6Aqg6NyJTeLYrrJr".to_string()];
    let images = fetcher.fetch_images(&uris).await.expect("fetch images");
    let image_raw_bytes = images.first().expect("image");
    let png = png_decoder!(image_raw_bytes);
    println!("image size: {:?}", png.len());
    // std::fs::write("dob1.png", &png).expect("write image");
}

#[tokio::test]
async fn test_dob1() {
    let settings = prepare_settings("text/plain");
    let decoder = DOBDecoder::new(settings);
    let (content, metadata) = generate_dob1_ingredients();
    let _output = decoder
        .decode_dna(&content["dna"].as_str().unwrap(), metadata)
        .await
        .expect("decode dob/1");
    // use base64::{engine::general_purpose::STANDARD, Engine};
    // let dob1_output: Value = serde_json::from_str(&_output).unwrap();
    // let base64_image = dob1_output
    //     .get("images")
    //     .unwrap()
    //     .as_array()
    //     .unwrap()
    //     .first()
    //     .unwrap()
    //     .as_object()
    //     .unwrap()
    //     .get("content")
    //     .unwrap()
    //     .as_str()
    //     .unwrap();
    // let image = STANDARD.decode(base64_image).expect("decode base64 image");
    // std::fs::write("dob1.png", &image).expect("write image");
}

#[tokio::test]
async fn test_fetch_dob1() {
    let settings = prepare_settings("text/plain");
    let decoder = DOBDecoder::new(settings);
    let dob1_spore_id =
        hex::decode("2fa44c408dfb78f5f032f6cb30966f9122df7e906c38bcbbd1fe9751ad3b2083").unwrap();
    let ((_, dna), metadata) = decoder
        .fetch_decode_ingredients(dob1_spore_id.try_into().unwrap())
        .await
        .unwrap();
    let output = decoder.decode_dna(&dna, metadata).await.unwrap();
    println!("dob1 output: {}", output);
    // use base64::{engine::general_purpose::STANDARD, Engine};
    // let dob1_output: Value = serde_json::from_str(&output).unwrap();
    // let base64_image = dob1_output
    //     .get("images")
    //     .unwrap()
    //     .as_array()
    //     .unwrap()
    //     .first()
    //     .unwrap()
    //     .as_object()
    //     .unwrap()
    //     .get("content")
    //     .unwrap()
    //     .as_str()
    //     .unwrap();
    // let image = STANDARD.decode(base64_image).expect("decode base64 image");
    // std::fs::write("dob1.png", &image).expect("write image");
}
