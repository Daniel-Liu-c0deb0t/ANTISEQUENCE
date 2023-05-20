use thiserror;

use std::fmt;

use crate::read::{StrType, Read};
use crate::inline_string::*;
use crate::fastq::Origin;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error reading or writing \"{file}\": {source}")]
    FileIo {
        file: String,
        source: Box<dyn std::error::Error>,
    },

    #[error("Error reading or writing bytes: {0}")]
    BytesIo(Box<dyn std::error::Error>),

    #[error("Unpaired read in {0}")]
    UnpairedRead(String),

    #[error("Error parsing record on line {line} in {origin}: {source}")]
    ParseRecord {
        origin: Origin,
        line: usize,
        source: Box<dyn std::error::Error>,
    },

    #[error("Could not parse \"{string}\" in \"{context}\": {reason}")]
    Parse {
        string: String,
        context: String,
        reason: &'static str,
    },

    #[error("Could not parse \"{string}\" in \"{context}\". Names must contain one or more alphanumeric characters, '_', or '*'.")]
    InvalidName {
        string: String,
        context: String,
    },

    #[error("Cannot find the {name} in the read:\n{read}\nwhen {context}")]
    NameNotInRead {
        name: Name,
        read: Read,
        context: &'static str,
    },

    #[error("Error parsing patterns:\n\"{patterns}\"\n{source}")]
    ParsePatterns {
        patterns: String,
        source: Box<dyn std::error::Error>,
    },
}

#[derive(thiserror::Error, Debug)]
#[error("Name not found in read: {0}")]
pub struct NameNotInReadError(pub Name);

#[derive(Debug)]
pub enum Name {
    StrType(StrType),
    Label(InlineString),
    Attr(InlineString),
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Name::*;
        match self {
            StrType(str_type) => write!(f, "string type \"{}\"", str_type),
            Label(label) => write!(f, "label \"{}\"", label),
            Attr(attr) => write!(f, "attribute \"{}\"", attr),
        }
    }
}

pub fn utf8(b: &[u8]) -> String {
    std::str::from_utf8(b).unwrap().to_owned()
}
