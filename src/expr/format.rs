pub struct FormatExpr {
    expr: Vec<Expr>,
}

enum Expr {
    Literal(String),
    Label(String),
    Data(String, String),
}

impl FormatExpr {
    pub fn new(expr: &str) -> Self {
        Self { expr: parse(expr) }
    }

    pub fn format(&self, read: &Read) -> String {
        let mut res = String::new();

        match self.expr {
            Literal(s) => res.push_str(&s),
            Label(label) => {
                let mapping = read.get_mapping(label).unwrap();
                res.push_str(std::str::from_utf8(read.get_region(mapping)).unwrap());
            }
            Data(label, attr) => {
                res.push_str(read.get_data(label, attr).unwrap().to_string());
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
                    [label] => Expr::Label(label.clone()),
                    [label, attr] => Expr::Data(lable.clone(), attr.clone()),
                    _ => panic!("Expected label or label.attr!"),
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
