use crate::expr;
use crate::inline_string::*;
use crate::read::*;

pub struct FormatExpr {
    expr: Vec<Expr>,
}

enum Expr {
    Literal(String),
    Label(expr::Label),
    Attr(expr::Attr),
}

impl FormatExpr {
    pub fn new(expr: &str) -> Self {
        Self { expr: parse(expr) }
    }

    pub fn format(&self, read: &Read, use_qual: bool) -> String {
        let mut res = String::new();

        for e in &self.expr {
            use Expr::*;
            match e {
                Literal(s) => res.push_str(&s),
                Label(expr::Label { str_type, label }) => {
                    let str_mappings = read.get_str_mappings(*str_type).unwrap();
                    let mapping = str_mappings.get_mapping(*label).unwrap();
                    let string = if use_qual {
                        str_mappings.substring_qual(mapping).unwrap()
                    } else {
                        str_mappings.substring(mapping)
                    };
                    res.push_str(std::str::from_utf8(string).unwrap());
                }
                Attr(expr::Attr {
                    str_type,
                    label,
                    attr,
                }) => {
                    res.push_str(
                        &read
                            .get_str_mappings(*str_type)
                            .unwrap()
                            .get_data(*label, *attr)
                            .unwrap()
                            .to_string(),
                    );
                }
            }
        }

        res
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
                res.push(match v.as_slice() {
                    &[str_type, label] => Expr::Label(expr::Label {
                        str_type: StrType::new(str_type),
                        label: InlineString::new(label),
                    }),
                    &[str_type, label, attr] => Expr::Attr(expr::Attr {
                        str_type: StrType::new(str_type),
                        label: InlineString::new(label),
                        attr: InlineString::new(attr),
                    }),
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
