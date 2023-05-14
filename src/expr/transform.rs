use crate::expr::{Label, LabelOrAttr};

#[derive(Debug, Clone)]
pub struct TransformExpr {
    before: Vec<Label>,
    after: Vec<Option<LabelOrAttr>>,
}

impl TransformExpr {
    pub fn new(expr: &[u8]) -> Self {
        let (before, after) = parse(expr);
        Self { before, after }
    }

    pub fn check_size(&self, before_size: usize, after_size: usize) {
        assert_eq!(before_size, self.before.len());
        assert_eq!(after_size, self.after.len());
    }

    pub fn check_same_str_type(&self) {
        let str_type = self.before[0].str_type;
        assert!(self.before.iter().all(|l| l.str_type == str_type));
        assert!(self.after.iter().all(|label_or_attr| label_or_attr
            .as_ref()
            .map(|l| l.str_type() == str_type)
            .unwrap_or(true)));
    }

    pub fn before(&self) -> &[Label] {
        &self.before
    }

    pub fn after(&self) -> &[Option<LabelOrAttr>] {
        &self.after
    }
}

fn parse(expr: &[u8]) -> (Vec<Label>, Vec<Option<LabelOrAttr>>) {
    let split_idx = expr
        .windows(2)
        .position(|w| w == b"->")
        .expect("Expected '->' in transform expression!");
    let before_str = &expr[..split_idx];
    let after_str = &expr[split_idx + 2..];

    let before_str = before_str
        .iter()
        .cloned()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<Vec<_>>();
    let before = before_str
        .split(|&b| b == b',')
        .map(|s| Label::new(s))
        .collect::<Vec<_>>();

    let after_str = after_str
        .iter()
        .cloned()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<Vec<_>>();
    let after = after_str
        .split(|&b| b == b',')
        .map(|s| {
            if s.iter().all(|&c| c == b'_') {
                None
            } else {
                Some(LabelOrAttr::new(s))
            }
        })
        .collect::<Vec<_>>();

    (before, after)
}
