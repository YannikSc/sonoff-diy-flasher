#[macro_use]
extern crate gotham_derive;
extern crate serde;

use std::env::args;
use std::env::current_dir;

use error::FlasherError;
use http::router;

use crate::hasher::sha256_hash_file;
use crate::sonoff_api::flash_sonoff;

mod error;
mod file_range;
mod hasher;
mod http;
mod range;
mod sonoff_api;

fn try_main() -> Result<(), FlasherError> {
    let bind_address = "127.0.0.1:8001";
    let mut app_args = args();

    app_args.next();
    let path = app_args.next().ok_or(FlasherError::new(
        "Argument 1 has to be a path to the firmware.",
    ))?;
    let sonoff_ip = app_args.next().ok_or(FlasherError::new(
        "Argument 2 has to be the IP for the sonoff.",
    ))?;
    let file = current_dir().unwrap().join(path);

    if !file.exists() || !file.is_file() {
        return Err(FlasherError::new(format!(
            "Path {:?} does not exists or is not a file",
            &file
        )));
    }

    let firmware_hash = sha256_hash_file(&file)?;
    println!("[info] starting firmware-serving web server");
    gotham::start_with_num_threads(bind_address.clone(), router(file), 1);

    println!("[info] flashing sonoff");
    flash_sonoff(&sonoff_ip, &String::from(bind_address), &firmware_hash)?;

    Ok(())
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("[error] {}", error);
    }
}
