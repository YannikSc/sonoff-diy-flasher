use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::FlasherError;

/// Used to wrap sonoff requests
///
/// # TODOS
///
/// TODO: Keep eye on generic. May need the trait bound to Serialize + DeserializeOwned ([related compiler bug](https://github.com/rust-lang/rust/issues/41617))
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SonoffRequestBody<D> {
    #[serde(rename = "deviceid")]
    pub device_id: String,
    pub data: D,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SonoffResponse<D> {
    pub data: D,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OtaFlashRequest {
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    #[serde(rename = "sha256sum")]
    pub sha256sum: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InfoResponse {
    pub ssid: String,
    #[serde(rename = "otaUnlock")]
    pub ota_unlock: bool,
    #[serde(rename = "fwVersion")]
    pub fw_version: String,
}

pub fn flash_sonoff(
    host: &String,
    own_bind: &String,
    firmware_sha256: &String,
) -> Result<(), FlasherError> {
    let result = get_sonoff_info(host)?;

    if !result.data.ota_unlock {
        println!("[info] unlocking sonoff ota mode");
        unlock_ota(host)?;
    }

    let result = get_sonoff_info(host)?;

    if !result.data.ota_unlock {
        return Err(FlasherError::new("Could not unlock ota mode"));
    }

    println!("[info] done: {:#?}", &result);
    println!("[info] flashing firmware. (this can take a while without response)");

    flash_ota(host, own_bind, firmware_sha256)?;

    println!("[info] flashed!");

    Ok(())
}

pub fn get_sonoff_info(host: &String) -> Result<SonoffResponse<InfoResponse>, FlasherError> {
    send_http_request(
        format!("http://{}:8081/zeroconf/info", host),
        SonoffRequestBody::<Value>::empty(),
    )
}

pub fn unlock_ota(host: &String) -> Result<SonoffResponse<InfoResponse>, FlasherError> {
    send_http_request(
        format!("http://{}:8081/zeroconf/ota_unlock", host),
        SonoffRequestBody::<Value>::empty(),
    )
}

pub fn flash_ota(
    host: &String,
    own_bind: &String,
    firmware_sha256: &String,
) -> Result<SonoffResponse<()>, FlasherError> {
    if let Err(error) = send_http_request::<OtaFlashRequest, ()>(
        format!("http://{}:8081/zeroconf/ota_flash", host),
        SonoffRequestBody::new(
            "",
            OtaFlashRequest {
                download_url: format!("http://{}/firmware.bin", own_bind),
                sha256sum: firmware_sha256.clone(),
            },
        ),
    ) {
        eprintln!("[warn] could not complete flashing request. This may not be an error! {}", error)
    }

    Ok(SonoffResponse { data: () })
}

pub fn send_http_request<D: Serialize + DeserializeOwned, O: DeserializeOwned>(
    url: String,
    body: SonoffRequestBody<D>,
) -> Result<SonoffResponse<O>, FlasherError> {
    let client = reqwest::blocking::Client::new();

    let response = client
        .post(url)
        .json(&body)
        .send()
        .or_else(|error| Err(FlasherError(format!("Could not send request: {}", error))))?;

    response.json().or_else(|error| {
        Err(FlasherError::new(format!(
            "Could not complete request: {}",
            error
        )))
    })
}

impl<D: Default> SonoffRequestBody<D> {
    pub fn empty() -> Self {
        Self {
            device_id: "".to_string(),
            data: Default::default(),
        }
    }
}

impl<D> SonoffRequestBody<D> {
    pub fn new(device_id: impl ToString, data: D) -> Self {
        Self {
            device_id: device_id.to_string(),
            data,
        }
    }
}
