use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::path::PathBuf;

use crate::error::FlasherError;

pub fn sha256_hash_file(firmware_path: &PathBuf) -> Result<String, FlasherError> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(false)
        .open(firmware_path)
        .or(Err(FlasherError(format!(
            "Could not open firmware file {:?}",
            firmware_path
        ))))?;

    let mut hasher = Sha256::new();

    std::io::copy(&mut file, &mut hasher)
        .or(Err(FlasherError::new("Could not copy file to hasher")))?;

    let result = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{:x}", byte))
        .collect();

    Ok(result)
}
