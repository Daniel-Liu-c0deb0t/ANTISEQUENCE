use crate::expr::Label;

pub struct TransformExpr {
    before: Vec<Label>,
    after: Vec<Label>,
}

impl TransformExpr {
    pub fn new(expr: &str) -> Self {
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
        assert!(self.after.iter().all(|l| l.str_type == str_type));
    }

    pub fn before(&self) -> &[Label] {
        &self.before
    }

    pub fn after(&self) -> &[Label] {
        &self.after
    }
}

fn parse(expr: &str) -> (Vec<Label>, Vec<Label>) {
    let mut split = expr.split("->");
    let before_str = split.next().unwrap();
    let after_str = split.next().unwrap();
    assert_eq!(split.next(), None);

    let before_str = before_str
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>();
    let before = before_str
        .split(',')
        .map(|s| Label::new(s))
        .collect::<Vec<_>>();

    let after_str = after_str
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>();
    let after = after_str
        .split(',')
        .map(|s| Label::new(s))
        .collect::<Vec<_>>();

    (before, after)
}
