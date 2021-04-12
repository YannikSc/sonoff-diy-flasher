use std::{
    fs::{File, OpenOptions},
    io::{Cursor, Read},
    path::PathBuf,
    time::SystemTime,
};

use rocket::{
    http::Header,
    response::{self, Responder},
    Request, Response,
};

use crate::range::{Position, Range, RangeBounding};
use rocket::http::Status;

pub enum RangedFileResult {
    One(RangedFile),
    Multiple(Vec<RangedFile>),
}

pub struct RangedFile {
    content: Vec<u8>,
    range: RangeBounding,
    file_size: u64,
}

pub fn read_file_ranges(range: Range, file: PathBuf) -> RangedFileResult {
    match range {
        Range::One(range) => RangedFileResult::One(read_file_range(&range, file.into())),
        Range::Multiple(ranges) => RangedFileResult::Multiple(
            ranges
                .iter()
                .map(|range| read_file_range(&range, file.clone().into()))
                .collect(),
        ),
    }
}

impl<'r> Responder<'r> for RangedFileResult {
    fn respond_to(self, _request: &Request) -> response::Result<'r> {
        match self {
            RangedFileResult::One(file) => {
                let size = get_header_range_from_file(&file);
                let mut response = Response::new();

                response.set_status(Status::new(206, "Partial Content"));
                response.set_sized_body(Cursor::new(file.content));
                response.set_header(Header::new("Content-Range", size));

                return Ok(response);
            }
            RangedFileResult::Multiple(files) => {
                let mut body: Vec<u8> = vec![];

                let boundary = format!("{}", SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs());

                for mut file in files {
                    body.append(&mut format!("\n--{}\n", &boundary).as_bytes().to_vec());
                    body.append(
                        &mut format!(
                            "Content-Type: application/octet-stream
Content-Range: {}\n\n",
                            get_header_range_from_file(&file)
                        )
                        .as_bytes()
                        .to_vec(),
                    );
                    body.append(&mut file.content);
                }

                body.append(&mut format!("\n--{}--\n", &boundary).as_bytes().to_vec());

                let mut response = Response::new();
                response.set_sized_body(Cursor::new(body));
                response.set_header(Header::new(
                    "Content-Type",
                    format!("multipart/byteranges; boundary={}", &boundary),
                ));

                return Ok(response);
            }
        }
    }
}

pub fn get_header_range_from_file(file: &RangedFile) -> String {
    let from = match file.range.from {
        Position::Fixed(value) => value,
        _ => 0_i64,
    };

    let mut to = match file.range.to {
        Position::Fixed(value) => value,
        _ => file.content.len() as i64,
    };

    if to > file.file_size as i64 {
        to = (file.file_size - 1) as i64;
    }

    println!("Serving {} {}-{}/{}", file.range.unit, from, to, file.file_size);

    format!("{} {}-{}/{}", file.range.unit, from, to, file.file_size)
}

fn read_file_range(range: &RangeBounding, file: PathBuf) -> RangedFile {
    let file = OpenOptions::new().write(false).read(true).open(file);

    if let Ok(file) = file {
        let size = get_file_size(&file);

        return RangedFile {
            content: extract_file_range(file, range),
            range: range.clone(),
            file_size: size,
        };
    }

    RangedFile {
        content: vec![],
        range: range.clone(),
        file_size: 0,
    }
}

fn get_file_size(file: &File) -> u64 {
    file.sync_all().unwrap_or_default();

    if let Ok(metadata) = file.metadata() {
        return metadata.len();
    }

    0
}

fn extract_file_range(file: File, range: &RangeBounding) -> Vec<u8> {
    let bytes = file
        .bytes()
        .skip(if let Position::Fixed(value) = &range.from {
            value.clone() as usize
        } else {
            0
        });

    let from = if let Position::Fixed(value) = &range.from {
        value.clone()
    } else {
        0_i64
    };

    if let Position::Fixed(value) = &range.to {
        let value = (value + 1) - from;

        return map_results(bytes.take(value.clone() as usize).collect());
    }

    map_results(bytes.collect())
}

fn map_results(input: Vec<Result<u8, std::io::Error>>) -> Vec<u8> {
    input
        .iter()
        .map(|byte| byte.as_ref().unwrap_or(&0_u8).clone())
        .collect()
}
