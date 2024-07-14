use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct PgnParser<'a> {
    tags: HashMap<&'a str, &'a str>,
}

impl<'a> PgnParser<'a> {
    pub fn new(pgn: &'a str) -> Result<PgnParseError, Self> {
        todo!()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PgnParseError {
    BadTagFormat,
}
