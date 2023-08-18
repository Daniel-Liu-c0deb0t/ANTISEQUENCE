use std::borrow::Cow;

use crate::errors::*;
use crate::inline_string::*;
use crate::read::*;
use crate::expr::*;

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

    pub fn from_exprs(patterns: Vec<Expr>) -> Self {
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
            attr_names: attr_names.into_iter().map(|v| InlineString::new(v.as_ref())).collect(),
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
    Expr { expr: Expr, attrs: Vec<Data> },
}

impl Pattern {
    pub fn get<'a>(&'a self, read: &'a Read) -> std::result::Result<Cow<'a, [u8]>, NameError> {
        use Pattern::*;
        match self {
            Literal { bytes, .. } => Ok(Cow::Borrowed(bytes)),
            Expr { expr, .. } => Ok(expr.eval_bytes(read, false)?),
        }
    }

    pub fn attrs(&self) -> &[Data] {
        use Pattern::*;
        match self {
            Literal { attrs, .. } => attrs,
            Expr { attrs, .. } => attrs,
        }
    }
}
