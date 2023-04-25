use crate::expr;
use crate::inline_string::*;
use crate::read::*;

pub struct SelectorExpr {
    expr: Expr,
}

impl SelectorExpr {
    pub fn new(expr_str: &str) -> Self {
        Self {
            expr: parse(&lex(expr_str)),
        }
    }

    pub fn matches(&self, read: &Read) -> bool {
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
    Label(String),
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

fn matches_rec(expr: &Expr, read: &Read) -> bool {
    use Expr::*;
    match expr {
        True => true,
        And(v) => v.iter().fold(true, |a, b| a & matches_rec(b, read)),
        Or(v) => v.iter().fold(false, |a, b| a | matches_rec(b, read)),
        Not(e) => !matches_rec(&e, read),
        Label(expr::Label { str_type, label }) => !read
            .get_str_mappings(*str_type)
            .unwrap()
            .get_mapping(*label)
            .unwrap()
            .is_empty(),
        Attr(expr::Attr {
            str_type,
            label,
            attr,
        }) => read
            .get_str_mappings(*str_type)
            .unwrap()
            .get_data(*label, *attr)
            .unwrap()
            .as_bool(),
    }
}

fn lex(expr_str: &str) -> Vec<Item> {
    let mut res = Vec::new();
    let mut curr = String::new();

    use Item::*;

    let write_curr = |res: &mut Vec<Item>, curr: &mut String, expect_empty| {
        assert!((expect_empty && curr.is_empty()) || (!expect_empty && !curr.is_empty()));

        if !curr.is_empty() {
            res.push(Label(curr.clone()));
            curr.clear();
        }
    };

    for c in expr_str.chars() {
        match c {
            '(' => {
                write_curr(&mut res, &mut curr, true);
                res.push(LeftParens);
            }
            ')' => {
                write_curr(&mut res, &mut curr, false);
                res.push(RightParens);
            }
            '&' => {
                write_curr(&mut res, &mut curr, false);
                res.push(And);
            }
            '|' => {
                write_curr(&mut res, &mut curr, false);
                res.push(Or);
            }
            '!' => {
                write_curr(&mut res, &mut curr, true);
                res.push(Not);
            }
            '.' => {
                write_curr(&mut res, &mut curr, false);
                res.push(Dot);
            }
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '*' => curr.push(c),
            ' ' | '\t' | '\n' | '\r' => (),
            _ => panic!("The character '{}' is not allowed!", c),
        }
    }

    if !curr.is_empty() {
        res.push(Label(curr.clone()));
    }

    res
}

fn parse(items: &[Item]) -> Expr {
    let items = unwrap_parens(items);

    if items.is_empty() {
        return Expr::True;
    }

    if items.len() == 3 {
        use Item::{Dot, Label};
        if let (Label(str_type), Dot, Label(label)) = (&items[0], &items[1], &items[2]) {
            return Expr::Label(expr::Label {
                str_type: StrType::new(&str_type),
                label: InlineString::new(label),
            });
        }
    }

    if items.len() == 5 {
        use Item::{Dot, Label};
        if let (Label(str_type), Dot, Label(label), Dot, Label(attr)) =
            (&items[0], &items[1], &items[2], &items[3], &items[4])
        {
            return Expr::Attr(expr::Attr {
                str_type: StrType::new(&str_type),
                label: InlineString::new(label),
                attr: InlineString::new(attr),
            });
        }
    }

    let mut exprs = Vec::new();

    let split = split_skip_parens(items, Item::Or, |curr_items| {
        exprs.push(parse(curr_items));
    });
    if split {
        return Expr::Or(exprs);
    }

    let split = split_skip_parens(items, Item::And, |curr_items| {
        exprs.push(parse(curr_items));
    });
    if split {
        return Expr::And(exprs);
    }

    if let Item::Not = items[0] {
        return Expr::Not(Box::new(parse(&items[1..])));
    } else {
        panic!("Expected unary NOT!");
    }
}

fn split_skip_parens<F>(items: &[Item], delim: Item, mut f: F) -> bool
where
    F: FnMut(&[Item]),
{
    let mut prev_idx = 0;
    let mut layer = 0;

    for (idx, item) in items.iter().enumerate() {
        use Item::*;
        match item {
            LeftParens => layer += 1,
            RightParens => {
                assert!(layer > 0, "Mismatched parentheses!");
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
        f(&items[prev_idx..items.len()]);
        true
    } else {
        false
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
