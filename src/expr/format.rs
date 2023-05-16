use crate::expr;
use crate::parse_utils::*;
use crate::read::*;

#[derive(Debug, Clone)]
pub struct FormatExpr {
    expr: Vec<Expr>,
}

#[derive(Debug, Clone)]
enum Expr {
    Literal(Vec<u8>),
    LabelOrAttr(expr::LabelOrAttr),
    Repeat(Box<Expr>, Num),
}

#[derive(Debug, Clone)]
enum Num {
    Literal(usize),
    LabelOrAttr(expr::LabelOrAttr),
}

impl FormatExpr {
    pub fn new(expr: &[u8]) -> Self {
        Self { expr: parse(expr) }
    }

    pub fn format(&self, read: &Read, use_qual: bool) -> Vec<u8> {
        let mut res = Vec::new();

        for e in &self.expr {
            format_expr(read, use_qual, e, &mut res);
        }

        res
    }
}

fn format_expr(read: &Read, use_qual: bool, e: &Expr, res: &mut Vec<u8>) {
    use Expr::*;
    match e {
        Literal(s) => res.extend(s),
        LabelOrAttr(l) => match l {
            expr::LabelOrAttr::Label(expr::Label { str_type, label }) => {
                let str_mappings = read.str_mappings(*str_type).unwrap();
                let mapping = str_mappings.mapping(*label).unwrap();
                let string = if use_qual {
                    str_mappings.substring_qual(mapping).unwrap()
                } else {
                    str_mappings.substring(mapping)
                };
                res.extend(string);
            }
            expr::LabelOrAttr::Attr(expr::Attr {
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
        },
        Repeat(expr, num) => {
            let repeats = match num {
                Num::Literal(n) => *n,
                Num::LabelOrAttr(l) => match l {
                    expr::LabelOrAttr::Label(expr::Label { str_type, label }) => {
                        read.str_mappings(*str_type)
                            .unwrap()
                            .mapping(*label)
                            .unwrap()
                            .len
                    }
                    expr::LabelOrAttr::Attr(expr::Attr {
                        str_type,
                        label,
                        attr,
                    }) => read
                        .str_mappings(*str_type)
                        .unwrap()
                        .data(*label, *attr)
                        .unwrap()
                        .as_uint(),
                },
            };

            if repeats >= 1 {
                let start = res.len();
                format_expr(read, use_qual, &*expr, res);
                let end = res.len();
                res.reserve((repeats - 1) * (end - start));

                for _ in 0..(repeats - 1) {
                    res.extend_from_within(start..end);
                }
            }
        }
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

                let idx = find_skip_quotes(&curr, b';');
                let end = idx.unwrap_or(curr.len());
                let left = trim_ascii_whitespace(&curr[..end]).unwrap();

                let e = if left[0] == b'\'' && left[left.len() - 1] == b'\'' {
                    Expr::Literal(left[1..left.len() - 1].to_owned())
                } else {
                    Expr::LabelOrAttr(expr::LabelOrAttr::new(left))
                };

                if let Some(idx) = idx {
                    let num_str = std::str::from_utf8(&curr[idx + 1..]).unwrap();
                    let num = num_str
                        .parse::<usize>()
                        .map(|n| Num::Literal(n))
                        .unwrap_or_else(|_| {
                            Num::LabelOrAttr(expr::LabelOrAttr::new(num_str.as_bytes()))
                        });

                    res.push(Expr::Repeat(Box::new(e), num));
                } else {
                    res.push(e);
                }

                in_label = false;
                curr.clear();
            }
            b'\\' if !escape => escape = true,
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
