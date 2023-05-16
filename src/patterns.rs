use serde::{Deserialize, Serialize};
use serde_yaml;

use std::collections::{BTreeMap, BTreeSet};

use crate::expr::FormatExpr;
use crate::inline_string::*;
use crate::read::*;

pub struct Patterns {
    pattern_name: InlineString,
    attr_names: Vec<InlineString>,
    patterns: Vec<Pattern>,
}

impl Patterns {
    pub fn new(name: InlineString, patterns: Vec<FormatExpr>) -> Self {
        Self {
            pattern_name: name,
            attr_names: Vec::new(),
            patterns: patterns
                .into_iter()
                .map(|v| Pattern {
                    expr: v,
                    attrs: Vec::new(),
                })
                .collect(),
        }
    }

    pub fn from_yaml(yaml: impl AsRef<[u8]>) -> Self {
        let patterns: PatternsSchema = serde_yaml::from_slice(yaml.as_ref()).unwrap();

        let pattern_name = InlineString::new(patterns.name.as_bytes());

        let attr_names = patterns.patterns[0]
            .attrs
            .iter()
            .map(|(k, _)| InlineString::new(k.as_bytes()))
            .collect::<BTreeSet<_>>();

        let patterns = patterns
            .patterns
            .into_iter()
            .map(|PatternSchema { pattern, attrs }| {
                let expr = FormatExpr::new(pattern.as_bytes());
                let attrs = attrs
                    .iter()
                    .map(|(k, v)| {
                        let s = InlineString::new(k.as_bytes());
                        assert!(attr_names.contains(&s));
                        v.to_data()
                    })
                    .collect::<Vec<_>>();
                Pattern { expr, attrs }
            })
            .collect::<Vec<_>>();

        let attr_names = attr_names.into_iter().collect::<Vec<_>>();

        Self {
            pattern_name,
            attr_names,
            patterns,
        }
    }

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
    pub expr: FormatExpr,
    pub attrs: Vec<Data>,
}

#[derive(Serialize, Deserialize)]
struct PatternsSchema {
    pub name: String,
    pub patterns: Vec<PatternSchema>,
}

#[derive(Serialize, Deserialize)]
struct PatternSchema {
    pub pattern: String,
    #[serde(flatten)]
    pub attrs: BTreeMap<String, DataSchema>,
}

#[derive(Serialize, Deserialize)]
enum DataSchema {
    Bool(bool),
    UInt(usize),
    String(String),
}

impl DataSchema {
    fn to_data(&self) -> Data {
        match self {
            DataSchema::Bool(x) => Data::Bool(*x),
            DataSchema::UInt(x) => Data::UInt(*x),
            DataSchema::String(x) => Data::Bytes(x.as_bytes().to_owned()),
        }
    }
}
