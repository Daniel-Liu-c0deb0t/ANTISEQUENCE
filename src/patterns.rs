use serde::{Deserialize, Serialize};
use serde_yaml;

use std::collections::{BTreeMap, BTreeSet};

use crate::errors::*;
use crate::inline_string::*;
use crate::read::*;

pub struct Patterns {
    pattern_name: Option<InlineString>,
    attr_names: Vec<InlineString>,
    patterns: Vec<Pattern>,
}

impl Patterns {
    pub fn from_strs(patterns: Vec<impl AsRef<[u8]>>) -> Self {
        Self {
            pattern_name: None,
            attr_names: Vec::new(),
            patterns: patterns
                .into_iter()
                .map(|v| Pattern::Literal {
                    bytes: v.as_ref().to_owned(),
                    attrs: Vec::new(),
                })
                .collect(),
        }
    }

    pub fn from_exprs(patterns: Vec<Node>) -> Self {
        Self {
            pattern_name: None,
            attr_names: Vec::new(),
            patterns: patterns
                .into_iter()
                .map(|v| Pattern::Expr {
                    expr: v,
                    attrs: Vec::new(),
                })
                .collect(),
        }
    }

    pub fn new(pattern_name: impl AsRef<[u8]>, attr_names: Vec<impl AsRef<[u8]>>, patterns: Vec<Pattern>) -> Self {
        Self {
            pattern_name: Some(InlineString::new(pattern_name.as_ref())),
            attr_names: attr_names.into_iter().map(|v| InlineString::new(v.as_ref().to_owned())).collect(),
            patterns,
        }
    }

    pub fn pattern_name(&self) -> Option<InlineString> {
        self.pattern_name
    }

    pub fn attr_names(&self) -> &[InlineString] {
        &self.attr_names
    }

    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }

    pub fn all_literals(&self) -> bool {
        for p in &self.patterns {
            if let Pattern::Expr { .. } = p {
                return false;
            }
        }

        true
    }
}

pub enum Pattern {
    Literal { bytes: Vec<u8>, attrs: Vec<Data> },
    Expr { expr: Node, attrs: Vec<Data> },
}

impl Pattern {
    pub fn get(read: &Read) -> Result<Cow<[u8]>, NameError> {
        match self {
            Literal { bytes, .. } => Cow::Borrowed(bytes),
            Expr { expr, .. } => Cow::Owned(expr.eval_bytes(read, false)?),
        }
    }

    pub fn attrs(&self) -> &[Data] {
        match self {
            Literal { attrs, .. } => attrs,
            Expr { attrs, .. } => attrs,
        }
    }
}
