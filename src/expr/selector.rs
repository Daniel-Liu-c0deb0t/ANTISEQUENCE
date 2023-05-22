use crate::expr;
use crate::inline_string::*;
use crate::read::*;
use crate::errors::*;

#[derive(Debug, Clone)]
pub struct SelectorExpr {
    expr: Expr,
}

impl SelectorExpr {
    pub fn new(expr_str: &[u8]) -> Result<Self> {
        Ok(Self {
            expr: parse(&lex(expr_str)?)?,
        })
    }

    pub fn matches(&self, read: &Read) -> std::result::Result<bool, NameError> {
        matches_rec(&self.expr, read)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Item {
    LeftParens,
    RightParens,
    And,
    Or,
    Not,
    Dot,
    Label(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    True,
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
    Label(expr::Label),
    Attr(expr::Attr),
}

fn matches_rec(expr: &Expr, read: &Read) -> std::result::Result<bool, NameError> {
    use Expr::*;
    match expr {
        True => Ok(true),
        And(v) => {
            let mut res = true;
            for e in v {
                res &= matches_rec(e, read)?;
            }
            Ok(res)
        }
        Or(v) => {
            let mut res = false;
            for e in v {
                res |= matches_rec(e, read)?;
            }
            Ok(res)
        }
        Not(e) => Ok(!(matches_rec(&e, read)?)),
        Label(expr::Label { str_type, label }) => Ok(read.str_mappings(*str_type).ok_or_else(|| NameError::NotInRead(Name::StrType(*str_type)))?.mapping(*label).is_some()),
        Attr(expr::Attr {
            str_type,
            label,
            attr,
        }) => Ok(read.data(*str_type, *label, *attr)?.as_bool()),
    }
}

fn lex(expr_str: &[u8]) -> Result<Vec<Item>> {
    let mut res = Vec::new();
    let mut curr = Vec::new();

    use Item::*;

    let write_curr = |res: &mut Vec<Item>, curr: &mut Vec<u8>, expect_empty| {
        if (expect_empty && !curr.is_empty()) || (!expect_empty && curr.is_empty()) {
            Err(Error::Parse { string: utf8(expr_str), context: utf8(expr_str), reason: "invalid boolean expression" })?;
        }

        if !curr.is_empty() {
            res.push(Label(curr.clone()));
            curr.clear();
        }

        Ok(())
    };

    for &c in expr_str {
        match c {
            b'(' => {
                write_curr(&mut res, &mut curr, true)?;
                res.push(LeftParens);
            }
            b')' => {
                write_curr(&mut res, &mut curr, false)?;
                res.push(RightParens);
            }
            b'&' => {
                write_curr(&mut res, &mut curr, false)?;
                res.push(And);
            }
            b'|' => {
                write_curr(&mut res, &mut curr, false)?;
                res.push(Or);
            }
            b'!' => {
                write_curr(&mut res, &mut curr, true)?;
                res.push(Not);
            }
            b'.' => {
                write_curr(&mut res, &mut curr, false)?;
                res.push(Dot);
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'*' => curr.push(c),
            _ if c.is_ascii_whitespace() => (),
            _ => Err(Error::Parse { string: (c as char).to_string() , context: utf8(expr_str), reason: "invalid character" })?,
        }
    }

    if !curr.is_empty() {
        res.push(Label(curr.clone()));
    }

    Ok(res)
}

fn parse(items: &[Item]) -> Result<Expr> {
    let items = unwrap_parens(items);

    if items.is_empty() {
        return Ok(Expr::True);
    }

    if items.len() == 3 {
        use Item::{Dot, Label};
        if let (Label(str_type), Dot, Label(label)) = (&items[0], &items[1], &items[2]) {
            return Ok(Expr::Label(expr::Label {
                str_type: StrType::new(&str_type)?,
                label: InlineString::new(label),
            }));
        }
    }

    if items.len() == 5 {
        use Item::{Dot, Label};
        if let (Label(str_type), Dot, Label(label), Dot, Label(attr)) =
            (&items[0], &items[1], &items[2], &items[3], &items[4])
        {
            return Ok(Expr::Attr(expr::Attr {
                str_type: StrType::new(&str_type)?,
                label: InlineString::new(label),
                attr: InlineString::new(attr),
            }));
        }
    }

    let mut exprs = Vec::new();

    let split = split_skip_parens(items, Item::Or, |curr_items| {
        exprs.push(parse(curr_items)?);
        Ok(())
    })?;
    if split {
        return Ok(Expr::Or(exprs));
    }

    let split = split_skip_parens(items, Item::And, |curr_items| {
        exprs.push(parse(curr_items)?);
        Ok(())
    })?;
    if split {
        return Ok(Expr::And(exprs));
    }

    if let Item::Not = items[0] {
        Ok(Expr::Not(Box::new(parse(&items[1..])?)))
    } else {
        Err(Error::Parse { string: "".to_owned(), context: "".to_owned(), reason: "invalid boolean expression" })
    }
}

fn split_skip_parens<F>(items: &[Item], delim: Item, mut f: F) -> Result<bool>
where
    F: FnMut(&[Item]) -> Result<()>,
{
    let mut prev_idx = 0;
    let mut layer = 0;

    for (idx, item) in items.iter().enumerate() {
        use Item::*;
        match item {
            LeftParens => layer += 1,
            RightParens => {
                if layer == 0 {
                    Err(Error::Parse { string: "".to_owned(), context: "".to_owned(), reason: "mismatched parentheses" })?;
                }
                layer -= 1;
            }
            _ if layer == 0 && item == &delim => {
                f(&items[prev_idx..idx]);
                prev_idx = idx + 1;
            }
            _ => (),
        }
    }

    if prev_idx > 0 {
        f(&items[prev_idx..items.len()])?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn unwrap_parens(items: &[Item]) -> &[Item] {
    let c1 = items.iter().take_while(|&i| i == &Item::LeftParens).count();
    let c2 = items
        .iter()
        .rev()
        .take_while(|&i| i == &Item::RightParens)
        .count();
    let c = c1.min(c2);
    &items[c..items.len() - c]
}
