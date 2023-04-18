pub struct FormatExpr {
    expr: Vec<Expr>,
}

enum Expr {
    Literal(String),
    Label(StrType, String),
    Data(StrType, String, String),
}

impl FormatExpr {
    pub fn new(expr: &str) -> Self {
        Self { expr: parse(expr) }
    }

    pub fn format(&self, read: &Read) -> String {
        let mut res = String::new();

        match self.expr {
            Literal(s) => res.push_str(&s),
            Label(str_type, label) => {
                let mapping = read.get_str_mappings(str_type).unwrap().get_mapping(label).unwrap();
                res.push_str(std::str::from_utf8(read.substring(mapping)).unwrap());
            }
            Data(str_type, label, attr) => {
                res.push_str(read.get_str_mappings(str_type).unwrap().get_data(label, attr).unwrap().to_string());
            }
        }
    }
}

fn parse(expr: &str) -> Vec<Expr> {
    let mut res = Vec::new();
    let mut curr = String::new();
    let mut escape = false;
    let mut in_label = false;

    for c in expr.chars() {
        match c {
            '{' if !escape => {
                assert!(!in_label);
                res.push(Expr::Literal(curr.clone()));
                in_label = true;
                curr.clear();
            }
            '}' if !escape => {
                assert!(in_label);

                let v = curr.split(".").collect::<Vec<_>>();
                res.push(match v {
                    [str_type, label] => Expr::Label(StrType::new(str_type), label.clone()),
                    [str_type, label, attr] => Expr::Data(StrType::new(str_type), label.clone(), attr.clone()),
                    _ => panic!("Expected type.label or type.label.attr!"),
                });

                in_label = false;
                curr.clear();
            }
            '\\' if !escape => escape = true,
            ' ' | '\t' | '\n' | '\r' if in_label => (),
            _ => {
                escape = false;
                curr.push(c);
            }
        }
    }

    if !curr.is_empty() {
        assert!(!in_label);
        res.push(Expr::Literal(curr));
    }

    res
}
