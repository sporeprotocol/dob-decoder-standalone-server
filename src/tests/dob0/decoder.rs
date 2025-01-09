use ckb_types::{h256, H256};
use serde_json::{json, Value};

use crate::decoder::DOBDecoder;
use crate::tests::prepare_settings;
use crate::types::{
    ClusterDescriptionField, DOBClusterFormat, DOBClusterFormatV0, DOBDecoderFormat,
    DecoderLocationType,
};

const EXPECTED_UNICORN_RENDER_RESULT: &str = "[{\"name\":\"wuxing_yinyang\",\"traits\":[{\"String\":\"3<_>\"}]},{\"name\":\"prev.bgcolor\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['#DBAB00', '#09D3FF', '#A028E9', '#FF3939', '#(135deg, #FE4F4F, #66C084, #00E2E2, #E180E2, #F4EC32)']\"}]},{\"name\":\"prev<%v>\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['#000000', '#000000', '#000000', '#000000', '#000000', '#FFFFFF', '#FFFFFF', '#FFFFFF', '#FFFFFF', '#FFFFFF'])\"}]},{\"name\":\"Spirits\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['Metal, Golden Body', 'Wood, Blue Body', 'Water, White Body', 'Fire, Red Body', 'Earth, Colorful Body']\"}]},{\"name\":\"Yin Yang\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['Yin, Long hair', 'Yin, Long hair', 'Yin, Long hair', 'Yin, Long hair', 'Yin, Long hair', 'Yang, Short Hair', 'Yang, Short Hair', 'Yang, Short Hair', 'Yang, Short Hair', 'Yang, Short Hair']\"}]},{\"name\":\"Talents\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['Guard<~>', 'Death<~>', 'Forget<~>', 'Curse<~>', 'Hermit<~>', 'Attack<~>', 'Revival<~>', 'Summon<~>', 'Prophet<~>', 'Crown<~>']\"}]},{\"name\":\"Horn\",\"traits\":[{\"String\":\"(%wuxing_yinyang):['Praetorian Horn', 'Hel Horn', 'Lethe Horn', 'Necromancer Horn', 'Lao Tsu Horn', 'Warrior Horn', 'Shaman Horn', 'Bard Horn', 'Sibyl Horn', 'Caesar Horn']\"}]},{\"name\":\"Wings\",\"traits\":[{\"String\":\"Sun Wings\"}]},{\"name\":\"Tail\",\"traits\":[{\"String\":\"Meteor Tail\"}]},{\"name\":\"Horseshoes\",\"traits\":[{\"String\":\"Silver Horseshoes\"}]},{\"name\":\"Destiny Number\",\"traits\":[{\"Number\":65321}]},{\"name\":\"Lucky Number\",\"traits\":[{\"Number\":35}]}]";
const EXPECTED_EXAMPLE_RENDER_RESULT: &str = "[{\"name\":\"Name\",\"traits\":[{\"String\":\"Ethan\"}]},{\"name\":\"Age\",\"traits\":[{\"Number\":23}]},{\"name\":\"Score\",\"traits\":[{\"Number\":136}]},{\"name\":\"DNA\",\"traits\":[{\"String\":\"0xaabbcc\"}]},{\"name\":\"URL\",\"traits\":[{\"String\":\"http://127.0.0.1:8090\"}]},{\"name\":\"Value\",\"traits\":[{\"Number\":13417386}]}]";

const UNICORN_SPORE_ID: H256 =
    h256!("0x4f7fb83a65dae9b95c21e55d5776a84f17bb6377681befeedb20a077ce1d8aad");
const EXAMPLE_SPORE_ID: H256 =
    h256!("0x683d0362a2e67d6edc80e3bf16136fae8a7fba21f6cb013931c5994c9ddb8d70");

fn generate_unicorn_dob_ingredients(onchain_decoder: bool) -> (Value, ClusterDescriptionField) {
    let unicorn_content = json!({
        "block_number": 120,
        "cell_id": 11844,
        "dna": "df4ffcb5e7a283ea7e6f09a504d0e256",
    });
    let decoder = if onchain_decoder {
        DOBDecoderFormat {
            location: DecoderLocationType::TypeId,
            hash: Some(h256!(
                "0x564870fab22ae50ac2bf1e986f21f34d5c9b50a30ec5c7bd5bf9f29aafb21a76"
            )),
            script: None,
        }
    } else {
        DOBDecoderFormat {
            location: DecoderLocationType::CodeHash,
            hash: Some(h256!(
                "0x32f29aba4b17f3d05bec8cec55d50ef86766fd0bf82fdedaa14269f344d3784a"
            )),
            script: None,
        }
    };
    let unicorn_metadata = ClusterDescriptionField {
            description: "Unicorns are the first series of digital objects generated based on time and space on CKB. Combining the Birth Time-location Determining Destiny Theory, Five Element Theory and YinYang Theory, it provide a special way for people to get Unicorn's on-chain DNA. Now all the seeds(DNAs) are on chain, and a magic world can expand.".to_string(),
            dob: DOBClusterFormat::new_dob0(DOBClusterFormatV0 {
                decoder,
                pattern: serde_json::from_str("[[\"wuxing_yinyang\",\"string\",0,1,\"options\",[\"0<_>\",\"1<_>\",\"2<_>\",\"3<_>\",\"4<_>\",\"5<_>\",\"6<_>\",\"7<_>\",\"8<_>\",\"9<_>\"]],[\"prev.bgcolor\",\"string\",1,1,\"options\",[\"(%wuxing_yinyang):['#DBAB00', '#09D3FF', '#A028E9', '#FF3939', '#(135deg, #FE4F4F, #66C084, #00E2E2, #E180E2, #F4EC32)']\"]],[\"prev<%v>\",\"string\",2,1,\"options\",[\"(%wuxing_yinyang):['#000000', '#000000', '#000000', '#000000', '#000000', '#FFFFFF', '#FFFFFF', '#FFFFFF', '#FFFFFF', '#FFFFFF'])\"]],[\"Spirits\",\"string\",3,1,\"options\",[\"(%wuxing_yinyang):['Metal, Golden Body', 'Wood, Blue Body', 'Water, White Body', 'Fire, Red Body', 'Earth, Colorful Body']\"]],[\"Yin Yang\",\"string\",4,1,\"options\",[\"(%wuxing_yinyang):['Yin, Long hair', 'Yin, Long hair', 'Yin, Long hair', 'Yin, Long hair', 'Yin, Long hair', 'Yang, Short Hair', 'Yang, Short Hair', 'Yang, Short Hair', 'Yang, Short Hair', 'Yang, Short Hair']\"]],[\"Talents\",\"string\",5,1,\"options\",[\"(%wuxing_yinyang):['Guard<~>', 'Death<~>', 'Forget<~>', 'Curse<~>', 'Hermit<~>', 'Attack<~>', 'Revival<~>', 'Summon<~>', 'Prophet<~>', 'Crown<~>']\"]],[\"Horn\",\"string\",6,1,\"options\",[\"(%wuxing_yinyang):['Praetorian Horn', 'Hel Horn', 'Lethe Horn', 'Necromancer Horn', 'Lao Tsu Horn', 'Warrior Horn', 'Shaman Horn', 'Bard Horn', 'Sibyl Horn', 'Caesar Horn']\"]],[\"Wings\",\"string\",7,1,\"options\",[\"Wind Wings\",\"Night Shadow Wings\",\"Lightning Wings\",\"Sun Wings\",\"Golden Wings\",\"Cloud Wings\",\"Morning Glow Wings\",\"Star Wings\",\"Spring Wings\",\"Moon Wings\",\"Angel Wings\"]],[\"Tail\",\"string\",8,1,\"options\",[\"Meteor Tail\",\"Rainbow Tail\",\"Willow Tail\",\"Phoenix Tail\",\"Sunset Shadow Tail\",\"Socrates Tail\",\"Dumbledore Tail\",\"Venus Tail\",\"Gaia Tail\"]],[\"Horseshoes\",\"string\",9,1,\"options\",[\"Ice Horseshoes\",\"Crystal Horseshoes\",\"Maple Horseshoes\",\"Flame Horseshoes\",\"Thunder Horseshoes\",\"Lotus Horseshoes\",\"Silver Horseshoes\"]],[\"Destiny Number\",\"number\",10,4,\"range\",[50000,100000]],[\"Lucky Number\",\"number\",14,1,\"range\",[1,49]]]").unwrap(),
            }),
        };
    (unicorn_content, unicorn_metadata)
}

fn generate_example_dob_ingredients(onchain_decoder: bool) -> (Value, ClusterDescriptionField) {
    let example_content = json!({
        "block_number": 120,
        "cell_id": 11844,
        "dna": "df4ffcb5e7a283ea7e6f09a504d0e256"
    });
    let decoder = if onchain_decoder {
        DOBDecoderFormat {
            location: DecoderLocationType::TypeId,
            hash: Some(h256!(
                "0x564870fab22ae50ac2bf1e986f21f34d5c9b50a30ec5c7bd5bf9f29aafb21a76"
            )),
            script: None,
        }
    } else {
        DOBDecoderFormat {
            location: DecoderLocationType::CodeHash,
            hash: Some(h256!(
                "0x32f29aba4b17f3d05bec8cec55d50ef86766fd0bf82fdedaa14269f344d3784a"
            )),
            script: None,
        }
    };
    let example_metadata = ClusterDescriptionField {
            description: "DOB/0 example.".to_string(),
            dob: DOBClusterFormat::new_dob0(DOBClusterFormatV0 {
                decoder,
                pattern: serde_json::from_str("[[\"Name\",\"string\",0,1,\"options\",[\"Alice\",\"Bob\",\"Charlie\",\"David\",\"Ethan\",\"Florence\",\"Grace\",\"Helen\"]],[\"Age\",\"number\",1,1,\"range\",[0,100]],[\"Score\",\"number\",2,1,\"raw\"],[\"DNA\",\"string\",3,3,\"raw\"],[\"URL\",\"string\",6,21,\"utf8\"],[\"Value\",\"number\",3,3,\"raw\"]]").unwrap(),
            }),
        };
    (example_content, example_metadata)
}

#[tokio::test]
async fn test_fetch_and_decode_unicorn_dna() {
    let settings = prepare_settings("text/plain");
    let decoder = DOBDecoder::new(settings);
    let (_, dna, dob_metadata) = decoder
        .fetch_decode_ingredients(UNICORN_SPORE_ID.into())
        .await
        .expect("fetch");
    let render_result = decoder
        .decode_dna(&dna, dob_metadata)
        // array type
        .await
        .expect("decode");
    assert_eq!(render_result, EXPECTED_UNICORN_RENDER_RESULT);
}

#[test]
fn test_unicorn_json_serde() {
    let (unicorn_content, unicorn_metadata) = generate_unicorn_dob_ingredients(false);
    let json_unicorn_content = serde_json::to_string(&unicorn_content).unwrap();
    let json_unicorn_metadata = serde_json::to_string(&unicorn_metadata).unwrap();
    println!("[spore_content] = {json_unicorn_content}");
    println!("[cluster_description] = {json_unicorn_metadata}");
    let deser_unicorn_content: Value =
        serde_json::from_slice(json_unicorn_content.as_bytes()).unwrap();
    let deser_unicorn_metadata: ClusterDescriptionField =
        serde_json::from_slice(json_unicorn_metadata.as_bytes()).unwrap();
    assert_eq!(unicorn_content, deser_unicorn_content);
    assert_eq!(unicorn_metadata, deser_unicorn_metadata);
}

#[tokio::test]
async fn test_fetch_and_decode_example_dna() {
    let settings = prepare_settings("text/plain");
    let decoder = DOBDecoder::new(settings);
    let (_, dna, dob_metadata) = decoder
        .fetch_decode_ingredients(EXAMPLE_SPORE_ID.into())
        .await
        .expect("fetch");
    let render_result = decoder
        .decode_dna(&dna, dob_metadata)
        // array type
        .await
        .expect("decode");
    assert_eq!(render_result, EXPECTED_EXAMPLE_RENDER_RESULT);
}

#[test]
fn test_example_json_serde() {
    let (content, metadata) = generate_example_dob_ingredients(false);
    let json_content = serde_json::to_string(&content).unwrap();
    let json_metadata = serde_json::to_string(&metadata).unwrap();
    println!("[spore_content] = {json_content}");
    println!("[cluster_description] = {json_metadata}");
}
