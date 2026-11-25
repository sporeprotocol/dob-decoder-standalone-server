use std::fs;

use crate::types::Settings;

mod dob0;
mod dob1;

#[allow(dead_code)]
pub enum SettingType {
    Mainnet,
    Testnet,
}

const TESTNET_SETTINGS_FILE: &str = "./settings.toml";
const MAINNET_SETTINGS_FILE: &str = "./settings.mainnet.toml";

fn prepare_settings(setting_type: SettingType, extra_version: Vec<&str>) -> Settings {
    let settings_file = match setting_type {
        SettingType::Mainnet => {
            fs::read_to_string(MAINNET_SETTINGS_FILE).expect("read settings.mainnet.toml")
        }
        SettingType::Testnet => {
            fs::read_to_string(TESTNET_SETTINGS_FILE).expect("read settings.toml")
        }
    };
    let mut settings: Settings = toml::from_str(&settings_file).unwrap();
    settings
        .protocol_versions
        .extend(extra_version.iter().map(|v| v.to_string()));
    settings
}
