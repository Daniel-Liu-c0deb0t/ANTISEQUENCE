use crate::errors::*;
use crate::expr::{Label, LabelOrAttr};
use crate::parse_utils::*;

#[derive(Debug, Clone)]
pub struct TransformExpr {
    before: Vec<Label>,
    after: Vec<Option<LabelOrAttr>>,
}

impl TransformExpr {
    pub fn new(expr: &[u8]) -> Result<Self> {
        let (before, after) = parse(expr)?;
        Ok(Self { before, after })
    }

    pub fn check_size(&self, before_size: usize, after_size: usize, context: &'static str) {
        assert_eq!(
            before_size,
            self.before.len(),
            "Number of labels before the \"->\" must be {}, only found {} labels for {}",
            before_size,
            self.before.len(),
            context
        );
        assert_eq!(after_size, self.after.len(), "Number of labels or attributes after the \"->\" must be {}, only found {} labels or attributes for {}", after_size, self.after.len(), context);
    }

    pub fn check_same_str_type(&self, context: &'static str) {
        let str_type = self.before[0].str_type;
        assert!(
            self.before.iter().all(|l| l.str_type == str_type),
            "String types before the \"->\" must be the same for {}",
            context
        );
        assert!(
            self.after.iter().all(|label_or_attr| label_or_attr
                .as_ref()
                .map(|l| l.str_type() == str_type)
                .unwrap_or(true)),
            "String types after the \"->\" must be the same for {}",
            context
        );
    }

    pub fn before(&self) -> &[Label] {
        &self.before
    }

    pub fn after(&self) -> &[Option<LabelOrAttr>] {
        &self.after
    }
}

fn parse(expr: &[u8]) -> Result<(Vec<Label>, Vec<Option<LabelOrAttr>>)> {
    let split_idx = expr
        .windows(2)
        .position(|w| w == b"->")
        .ok_or_else(|| Error::Parse {
            string: utf8(expr),
            context: utf8(expr),
            reason: "missing \"->\"",
        })?;
    let before_str = expr[..split_idx].to_owned();
    let after_str = expr[split_idx + 2..].to_owned();

    let before = before_str
        .split(|&b| b == b',')
        .map(|s| Label::new(s))
        .collect::<Result<Vec<_>>>()?;

    let after = after_str
        .split(|&b| b == b',')
        .map(|s| {
            let s = trim_ascii_whitespace(s).ok_or_else(|| Error::InvalidName {
                string: utf8(s),
                context: utf8(expr),
            })?;

            if s.iter().all(|&c| c == b'_') {
                Ok(None)
            } else {
                Ok(Some(LabelOrAttr::new(s)?))
            }
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((before, after))
}
