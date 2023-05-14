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
    Repeat(Box<Expr>, Num),
}

#[derive(Debug, Clone)]
enum Num {
    Literal(usize),
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
            format_expr(read, use_qual, e, &mut res);
        }

        res
    }
}

fn format_expr(read: &Read, use_qual: bool, e: &Expr, res: &mut Vec<u8>) {
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
        Repeat(expr, num) => {
            let repeats = match num {
                Num::Literal(n) => *n,
                Num::Label(expr::Label { str_type, label }) => {
                    read.str_mappings(*str_type)
                        .unwrap()
                        .mapping(*label)
                        .unwrap()
                        .len
                }
                Num::Attr(expr::Attr {
                    str_type,
                    label,
                    attr,
                }) => read
                    .str_mappings(*str_type)
                    .unwrap()
                    .data(*label, *attr)
                    .unwrap()
                    .as_uint(),
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
                let left = &curr[..end];

                let e = if left[0] == b'\'' && left[left.len() - 1] == b'\'' {
                    Expr::Literal(left[1..left.len() - 1].to_owned())
                } else {
                    let v = curr[..end].split(|&b| b == b'.').collect::<Vec<_>>();
                    match v.as_slice() {
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
                    }
                };

                if let Some(idx) = idx {
                    let num_str = std::str::from_utf8(&curr[idx + 1..]).unwrap();
                    let num = num_str
                        .parse::<usize>()
                        .map(|n| Num::Literal(n))
                        .unwrap_or_else(|_| {
                            let v = num_str.as_bytes().split(|&b| b == b'.').collect::<Vec<_>>();
                            match v.as_slice() {
                                &[str_type, label] => Num::Label(expr::Label {
                                    str_type: StrType::new(str_type),
                                    label: InlineString::new(label),
                                }),
                                &[str_type, label, attr] => Num::Attr(expr::Attr {
                                    str_type: StrType::new(str_type),
                                    label: InlineString::new(label),
                                    attr: InlineString::new(attr),
                                }),
                                _ => panic!("Expected type.label or type.label.attr!"),
                            }
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

fn find_skip_quotes(s: &[u8], c: u8) -> Option<usize> {
    let mut escape = false;
    let mut in_quotes = false;

    for (i, &b) in s.iter().enumerate() {
        match b {
            b'\'' if !escape && !in_quotes => in_quotes = true,
            b'\'' if !escape && in_quotes => in_quotes = false,
            b'\\' if !escape => escape = true,
            _ if !in_quotes && b == c => return Some(i),
            _ => escape = false,
        }
    }

    None
}
