use crate::expr;
use crate::inline_string::*;
use crate::read::*;

#[derive(Debug, Clone)]
pub struct FormatExpr {
    expr: Vec<Expr>,
}

#[derive(Debug, Clone)]
enum Expr {
    Literal(Vec<u8>),
    Label(expr::Label),
    Attr(expr::Attr),
}

impl FormatExpr {
    pub fn new(expr: &[u8]) -> Self {
        Self { expr: parse(expr) }
    }

    pub fn format(&self, read: &Read, use_qual: bool) -> Vec<u8> {
        let mut res = Vec::new();

        for e in &self.expr {
            use Expr::*;
            match e {
                Literal(s) => res.extend(s),
                Label(expr::Label { str_type, label }) => {
                    let str_mappings = read.str_mappings(*str_type).unwrap();
                    let mapping = str_mappings.mapping(*label).unwrap();
                    let string = if use_qual {
                        str_mappings.substring_qual(mapping).unwrap()
                    } else {
                        str_mappings.substring(mapping)
                    };
                    res.extend(string);
                }
                Attr(expr::Attr {
                    str_type,
                    label,
                    attr,
                }) => {
                    res.extend(
                        read.str_mappings(*str_type)
                            .unwrap()
                            .data(*label, *attr)
                            .unwrap()
                            .to_string()
                            .as_bytes(),
                    );
                }
            }
        }

        res
    }
}

fn parse(expr: &[u8]) -> Vec<Expr> {
    let mut res = Vec::new();
    let mut curr = Vec::new();
    let mut escape = false;
    let mut in_label = false;

    for &c in expr {
        match c {
            b'{' if !escape => {
                assert!(!in_label);
                res.push(Expr::Literal(curr.clone()));
                in_label = true;
                curr.clear();
            }
            b'}' if !escape => {
                assert!(in_label);

                let v = curr.split(|&b| b == b'.').collect::<Vec<_>>();
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
            b'\\' if !escape => escape = true,
            _ if c.is_ascii_whitespace() && in_label => (),
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
