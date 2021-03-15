#[derive(Clone, Debug)]
pub enum Range {
    One(RangeBounding),
    Multiple(Vec<RangeBounding>),
}

#[derive(Clone, Debug)]
pub enum Position {
    Start,
    End,
    Fixed(i64),
}

#[derive(Clone, Debug)]
pub struct RangeBounding {
    pub unit: String,
    pub from: Position,
    pub to: Position,
}

pub fn parse_range_header(raw_range: String) -> Range {
    let mut split = raw_range.split('=');
    let unit = String::from(split.next().unwrap_or("bytes"));
    let ranges = String::from(split.next().unwrap_or(""));

    let mut parsed_ranges: Vec<RangeBounding> = vec![];

    for range in ranges.split(",") {
        parsed_ranges.push(parse_single_range(String::from(range), unit.clone()));
    }

    if parsed_ranges.len() == 1 {
        return Range::One(parsed_ranges.remove(0));
    }

    Range::Multiple(parsed_ranges)
}

fn parse_single_range(range: String, unit: String) -> RangeBounding {
    let mut split = range.split('-');
    let from = if let Ok(value) = String::from(split.next().unwrap_or("")).parse() {
        Position::Fixed(value)
    } else {
        Position::Start
    };

    let to = if let Ok(value) = String::from(split.next().unwrap_or("")).parse() {
        Position::Fixed(value)
    } else {
        Position::End
    };

    RangeBounding { unit, from, to }
}
