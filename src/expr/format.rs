use crate::errors::*;
use crate::expr;
use crate::parse_utils::*;
use crate::read::*;

const UNKNOWN_QUAL: u8 = b'I';

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
    LabelOrAttrLen(expr::LabelOrAttr),
    LabelOrAttrCoerce(expr::Attr),
}

impl FormatExpr {
    pub fn new(expr: &[u8]) -> Result<Self> {
        Ok(Self { expr: parse(expr)? })
    }

    pub fn format(&self, read: &Read, use_qual: bool) -> std::result::Result<Vec<u8>, NameError> {
        let mut res = Vec::new();

        for e in &self.expr {
            format_expr(read, use_qual, e, &mut res)?;
        }

        Ok(res)
    }
}

fn format_expr(
    read: &Read,
    use_qual: bool,
    e: &Expr,
    res: &mut Vec<u8>,
) -> std::result::Result<(), NameError> {
    use Expr::*;
    match e {
        Literal(s) => res.extend(s),
        LabelOrAttr(l) => match l {
            expr::LabelOrAttr::Label(expr::Label { str_type, label }) => {
                if use_qual {
                    if let Some(qual) = read.substring_qual(*str_type, *label)? {
                        res.extend(qual);
                    } else {
                        let len = read.mapping(*str_type, *label)?.len;
                        res.extend((0..len).map(|_| UNKNOWN_QUAL));
                    }
                } else {
                    res.extend(read.substring(*str_type, *label)?);
                }
            }
            expr::LabelOrAttr::Attr(expr::Attr {
                str_type,
                label,
                attr,
            }) => {
                res.extend(read.data(*str_type, *label, *attr)?.to_string().as_bytes());
            }
        },
        Repeat(expr, num) => {
            let repeats = match num {
                Num::Literal(n) => *n,
                Num::LabelOrAttrLen(l) => match l {
                    expr::LabelOrAttr::Label(expr::Label { str_type, label }) => {
                        read.mapping(*str_type, *label)?.len
                    }
                    expr::LabelOrAttr::Attr(expr::Attr {
                        str_type,
                        label,
                        attr,
                    }) => read.data(*str_type, *label, *attr)?.len()?,
                },
                Num::LabelOrAttrCoerce(expr::Attr {
                    str_type,
                    label,
                    attr,
                }) => read.data(*str_type, *label, *attr)?.as_int()? as usize,
            };

            if repeats >= 1 {
                let start = res.len();
                format_expr(read, use_qual, &*expr, res)?;
                let end = res.len();
                res.reserve((repeats - 1) * (end - start));

                for _ in 0..(repeats - 1) {
                    res.extend_from_within(start..end);
                }
            }
        }
    }

    Ok(())
}

fn parse(expr: &[u8]) -> Result<Vec<Expr>> {
    let mut res = Vec::new();
    let mut curr = Vec::new();
    let mut escape = false;
    let mut in_label = false;

    for &c in expr {
        match c {
            b'{' if !escape => {
                if in_label {
                    Err(Error::Parse {
                        string: utf8(expr),
                        context: utf8(expr),
                        reason: "cannot have nested braces",
                    })?;
                }
                res.push(Expr::Literal(curr.clone()));
                in_label = true;
                curr.clear();
            }
            b'}' if !escape => {
                if !in_label {
                    Err(Error::Parse {
                        string: utf8(expr),
                        context: utf8(expr),
                        reason: "unbalanced braces",
                    })?;
                }

                let idx = find_skip_quotes(&curr, b';');
                let end = idx.unwrap_or(curr.len());
                let left =
                    trim_ascii_whitespace(&curr[..end]).ok_or_else(|| Error::InvalidName {
                        string: utf8(&curr[..end]),
                        context: utf8(expr),
                    })?;

                let e = if left[0] == b'\'' && left[left.len() - 1] == b'\'' {
                    Expr::Literal(left[1..left.len() - 1].to_owned())
                } else {
                    Expr::LabelOrAttr(expr::LabelOrAttr::new(left)?)
                };

                if let Some(idx) = idx {
                    let right = trim_ascii_whitespace(&curr[idx + 1..]).ok_or_else(|| {
                        Error::InvalidName {
                            string: utf8(&curr[idx + 1..]),
                            context: utf8(expr),
                        }
                    })?;
                    let num_str = std::str::from_utf8(right).unwrap();
                    let num = num_str
                        .parse::<usize>()
                        .map(|n| Ok(Num::Literal(n)))
                        .unwrap_or_else(|_| {
                            if right[0] == b'|' && right[right.len() - 1] == b'|' {
                                Ok(Num::LabelOrAttrLen(expr::LabelOrAttr::new(
                                    &right[1..right.len() - 1],
                                )?))
                            } else {
                                Ok(Num::LabelOrAttrCoerce(expr::Attr::new(right)?))
                            }
                        })?;

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
        if in_label {
            Err(Error::Parse {
                string: utf8(expr),
                context: utf8(expr),
                reason: "unbalanced braces",
            })?;
        }
        res.push(Expr::Literal(curr));
    }

    Ok(res)
}
