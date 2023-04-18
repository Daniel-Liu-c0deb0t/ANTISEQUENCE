pub struct TransformExpr {
    before: Vec<Label>,
    after: Vec<Label>,
}

pub struct Label {
    pub str_type: StrType,
    pub label: String,
}

impl TransformExpr {
    pub fn new(expr: &str) -> Self {
        let (before, after) = parse(expr);
        Self { before, after }
    }

    pub fn check(&self, before_size: usize, after_size: usize) {
        assert_eq!(before_size, self.before.len());
        assert_eq!(after_size, self.after.len());
    }

    pub fn before(&self) -> &[String] {
        &self.before
    }

    pub fn after(&self) -> &[String] {
        &self.after
    }
}

fn parse(expr: &str) -> (Vec<Label>, Vec<Label>) {
    let mut split = expr.split("->");
    let before_str = split.next().unwrap();
    let after_str = split.next().unwrap();
    assert_eq!(split.next(), None);

    let before_str = before_str.chars().filter(|c| !c.is_ascii_whitespace()).collect::<String>();
    let before = before_str.split(',').map(|s| {
        let split = s.split('.').collect::<Vec<_>>();
        match split {
            [str_type, label] => Label { str_type: StrType::new(str_type), label: label.clone() },
            _ => panic!("Expected type.label!"),
        }
    }).collect::<Vec<_>>();

    let after_str = after_str.chars().filter(|c| !c.is_ascii_whitespace()).collect::<String>();
    let after = after_str.split(',').map(|s| {
        let split = s.split('.').collect::<Vec<_>>();
        match split {
            [str_type, label] => Label { str_type: StrType::new(str_type), label: label.clone() },
            _ => panic!("Expected type.label!"),
        }
    }).collect::<Vec<_>>();

    (before, after)
}
