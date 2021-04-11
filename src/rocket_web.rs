use std::{net::ToSocketAddrs, path::PathBuf};

use rocket::{request::FromRequest, routes, Config, Outcome, State};

use crate::{
    file_range::{read_file_ranges, RangedFileResult},
    range::{parse_range_header, Range},
};

use crate::error::FlasherError;

pub struct ServerResources {
    pub path: PathBuf,
}

struct RangeHeader {
    ranges: Range,
}

impl<'a, 'r> FromRequest<'a, 'r> for RangeHeader {
    type Error = ();

    fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let range = request.headers().get_one("Range").unwrap_or_default();
        let ranges = parse_range_header(range.into());

        Outcome::Success(RangeHeader { ranges })
    }
}

#[get("/firmware.bin")]
fn serve_firmware(range: RangeHeader, settings: State<ServerResources>) -> RangedFileResult {
    let _path = settings.path.clone();

    read_file_ranges(range.ranges, settings.path.clone())
}

pub fn launch(
    bind_address: impl ToSocketAddrs,
    firmware_path: PathBuf,
) -> Result<(), FlasherError> {
    let res = ServerResources {
        path: firmware_path,
    };
    let mut config = Config::development();

    let mut addrs = bind_address
        .to_socket_addrs()
        .or(Err(FlasherError::new("Could not parse bind address")))?;
    let addr = addrs
        .next()
        .ok_or(FlasherError::new("No addresses given"))?;

    config
        .set_address(addr.ip().to_string())
        .or(Err(FlasherError::new("Could net set address")))?;
    config.set_port(config.port + 1);

    rocket::custom(config)
        .manage(res)
        .mount("/", routes![serve_firmware])
        .launch();

    Ok(())
}
