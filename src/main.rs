#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate serde;

use std::env::args;
use std::env::current_dir;
use std::io::stdin;

use error::FlasherError;

use crate::hasher::sha256_hash_file;
use crate::sonoff_api::flash_sonoff;

mod error;
mod file_range;
mod hasher;
mod range;
mod sonoff_api;

mod rocket_web;

fn get_local_ip() -> Result<String, FlasherError> {
    let interfaces = get_if_addrs::get_if_addrs()
        .or(Err(FlasherError::new("Could not determine interface IPs.")))?;

    println!("[info] Select the IP on which the sonoff can reach your PC:");

    for index in 0..interfaces.len() {
        let interface = interfaces.get(index).unwrap();

        println!("[info] {}: {}", index, interface.addr.ip());
    }

    loop {
        let mut selected = String::new();

        eprint!("[input]> ");
        stdin()
            .read_line(&mut selected)
            .or(Err(FlasherError::new("Could not read from stdin")))?;

        if let Ok(number) = selected.trim().parse::<u32>() {
            if let Some(interface) = interfaces.get(number as usize) {
                let ip = interface.addr.ip().to_string();

                println!("[info] Selected {}", &ip);

                return Ok(ip);
            }
        }

        eprintln!(
            "[error] Insert a number between 0 and {}",
            interfaces.len() - 1
        );
    }
}

fn try_main() -> Result<(), FlasherError> {
    let bind_address = format!("{}:8001", get_local_ip()?);
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
    println!(
        "[info] starting firmware-serving web server on {}",
        &bind_address
    );

    println!("[info] flashing sonoff");

    match flash_sonoff(
        &sonoff_ip,
        &String::from(bind_address.clone()),
        &firmware_hash,
    ) {
        _ => {
            println!("[info] flashing in progress. This may take a while! Its done when the output stops.");
        }
    }

    rocket_web::launch(bind_address.clone(), file)?;

    Ok(())
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("[error] {}", error);
    }
}
