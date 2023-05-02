use crate::inline_string::*;
use crate::read::*;

pub struct Patterns {
    pattern_name: InlineString,
    attr_names: Vec<InlineString>,
    patterns: Vec<Pattern>,
}

impl Patterns {
    pub fn new(name: InlineString, patterns: Vec<Vec<u8>>) -> Self {
        Self {
            pattern_name: name,
            attr_names: Vec::new(),
            patterns: patterns
                .into_iter()
                .map(|v| Pattern {
                    pattern: v,
                    attrs: Vec::new(),
                })
                .collect(),
        }
    }

    pub fn from_tsv(tsv: &[u8]) -> Self {}

    pub fn pattern_name(&self) -> InlineString {
        self.pattern_name
    }

    pub fn attr_names(&self) -> &[InlineString] {
        &self.attr_names
    }

    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }
}

pub struct Pattern {
    pub pattern: Vec<u8>,
    pub attrs: Vec<Data>,
}
