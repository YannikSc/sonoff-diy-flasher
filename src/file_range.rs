use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::PathBuf,
    time::SystemTime,
};

use gotham::{
    handler::IntoResponse,
    hyper::{header::HeaderValue, Body, Response},
    state::State,
};

use crate::range::{Position, Range, RangeBounding};

pub enum RangedFileResult {
    One(RangedFile),
    Multiple(Vec<RangedFile>),
}

pub struct RangedFile {
    content: Vec<u8>,
    range: RangeBounding,
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

impl IntoResponse for RangedFileResult {
    fn into_response(self, _state: &State) -> Response<Body> {
        match self {
            RangedFileResult::One(file) => Response::new(file.content.into()),
            RangedFileResult::Multiple(files) => {
                let mut body: Vec<u8> = vec![];

                let boundary = format!("{}", SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs());

                for mut file in files {
                    body.append(&mut format!("\n--{}\n", &boundary).as_bytes().to_vec());
                    body.append(
                        &mut format!(
                            "Content-Type: application/octet-stream
Content-Range: bytes {}/32\n\n",
                            get_header_range_from_file(&file)
                        )
                        .as_bytes()
                        .to_vec(),
                    );
                    body.append(&mut file.content);
                }

                body.append(&mut format!("\n--{}--\n", &boundary).as_bytes().to_vec());

                let mut res = Response::new(body.into());

                res.headers_mut().insert(
                    "Content-Type",
                    HeaderValue::from_str(
                        format!("multipart/byteranges; boundary={}", &boundary).as_str(),
                    )
                    .unwrap(),
                );

                res
            }
        }
    }
}

fn get_header_range_from_file(file: &RangedFile) -> String {
    let from = match file.range.from {
        Position::Fixed(value) => value,
        _ => 0_i64,
    };

    let to = match file.range.from {
        Position::Fixed(value) => value,
        _ => file.content.len() as i64,
    };

    format!("{}-{}", from, to)
}

fn read_file_range(range: &RangeBounding, file: PathBuf) -> RangedFile {
    let file = OpenOptions::new().write(false).read(true).open(file);

    if let Ok(file) = file {
        return RangedFile {
            content: extract_file_range(file, range),
            range: range.clone(),
        };
    }

    RangedFile {
        content: vec![],
        range: range.clone(),
    }
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
