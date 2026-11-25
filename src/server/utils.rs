use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::types::Error;

pub fn trim_0x(hexed: &str) -> &str {
    hexed.trim_start_matches("0x")
}

pub fn read_dob_from_cache(
    cache_path: PathBuf,
    mut expiration: u64,
) -> Result<Option<(String, Value)>, Error> {
    if !cache_path.exists() {
        return Ok(None);
    }
    let file_content = fs::read_to_string(&cache_path)
        .map_err(|_| Error::DOBRenderCacheNotFound(cache_path.clone()))?;
    let mut lines = file_content.split('\n');
    let (Some(result), Some(content), timestamp) = (lines.next(), lines.next(), lines.next())
    else {
        return Err(Error::DOBRenderCacheModified(cache_path));
    };
    if let Some(value) = timestamp {
        if !value.is_empty() {
            expiration = value
                .parse::<u64>()
                .map_err(|_| Error::DOBRenderCacheModified(cache_path.clone()))?;
        }
    }
    match serde_json::from_str(content) {
        Ok(content) => {
            if expiration > 0 && now()? > Duration::from_secs(expiration) {
                Ok(None)
            } else {
                Ok(Some((result.to_string(), content)))
            }
        }
        Err(_) => Err(Error::DOBRenderCacheModified(cache_path)),
    }
}

pub fn write_dob_to_cache(
    render_result: &str,
    dob_content: &Value,
    cache_path: PathBuf,
    cache_expiration: u64,
) -> Result<(), Error> {
    let expiration_timestamp = if cache_expiration > 0 {
        now()?
            .checked_add(Duration::from_secs(cache_expiration))
            .ok_or(Error::SystemTimeError)?
            .as_secs()
    } else {
        0 // zero means always read from cache
    };
    let json_dob_content = serde_json::to_string(dob_content).unwrap();
    let file_content = format!("{render_result}\n{json_dob_content}\n{expiration_timestamp}");
    fs::write(&cache_path, file_content).map_err(|_| Error::DOBRenderCacheNotFound(cache_path))?;
    Ok(())
}

pub fn now() -> Result<Duration, Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::SystemTimeError)
}
