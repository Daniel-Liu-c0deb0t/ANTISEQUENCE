pub struct FormatExpr {
    expr: Vec<Expr>,
}

enum Expr {
    Literal(String),
    Label(String),
}

impl FormatExpr {
    pub fn new(expr: &str) -> Self {
        parse(expr)
    }

    pub fn format(&self, read: &Read) -> String {
        let mut res = String::new();

        match self.expr {
            Literal(s) => res.push_str(&s),
            Label(s) => {

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
                res.push(Expr::Label(curr.clone()));
                in_label = false;
                curr.clear();
            }
            '\\' if !escape => (),
            ' ' | '\t' | '\n' | '\r' if in_label => (),
            _ => curr.push(c),
        }

        if escape {
            escape = false;
        } else {
            escape = c == '\\';
        }
    }

    if !curr.is_empty() {
        assert!(!in_label);
        res.push(Expr::Literal(curr));
    }

    res
}
