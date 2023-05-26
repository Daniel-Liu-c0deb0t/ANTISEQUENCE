use thiserror;

use std::fmt;

use crate::fastq::Origin;
use crate::inline_string::*;
use crate::read::{Data, Read, StrType};

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
    InvalidName { string: String, context: String },

    #[error("{source}\nwith read:\n{read}when {context}")]
    NameError {
        source: NameError,
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
pub enum NameError {
    #[error("Name not found in read: {0}")]
    NotInRead(Name),
    #[error("Duplicate name in read: {0}")]
    Duplicate(Name),
    #[error("Expected {0}, but found {1:?}")]
    Type(&'static str, Data),
}

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
