use crate::read::*;

pub struct SelectorExpr {
    expr: Expr,
}

impl SelectorExpr {
    pub fn new(expr_str: &str) -> Self {
        Self { expr: parse(&lex(expr_str)) }
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
    Literal(String),
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Expr),
    Literal(String),
}

fn matches_rec(expr: &Expr, read: &Read) -> bool {
    use Expr::*;
    match expr {
        And(v) => v.iter().fold(true, |a, b| a & matches_rec(b, read)),
        Or(v) => v.iter().fold(false, |a, b| a | matches_rec(b, read)),
        Not(e) => !matches_rec(&e, read),
        Literal(label) => !read.get(label).unwrap().is_empty(),
    }
}

fn lex(expr_str: &str) -> Vec<Item> {
    let mut res = Vec::new();
    let mut curr = String::new();

    use Item::*;

    let mut write_curr = |expect_empty| {
        assert!((expect_empty && curr.is_empty()) || (!expect_empty && !curr.is_empty()));

        if !curr.is_empty() {
            res.push(Literal(curr.clone()));
            curr.clear();
        }
    };

    for c in expr_str.chars() {
        match c {
            '(' => {
                write_curr(true);
                res.push(LeftParens);
            }
            ')' => {
                write_curr(false);
                res.push(RightParens);
            }
            '&' => {
                write_curr(false);
                res.push(And);
            }
            '|' => {
                write_curr(false);
                res.push(Or);
            }
            '!' => {
                write_curr(true);
                res.push(Not);
            }
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' => curr.push(c),
            _ => panic!("The character '{}' is not allowed!", c);
        }
    }

    if !curr.is_empty() {
        res.push(Literal(curr.clone()));
    }

    res
}

fn parse(items: &[Item]) -> Expr {
    items = unwrap_parens(items);

    assert!(!items.is_empty(), "Expected non-empty expression!");

    if items.len() == 1 {
        if let Literal(label) = items[0] {
            return Expr::Literal(label.clone());
        } else {
            panic!("Expected literal label!");
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
        return Expr::Not(parse(&items[1..]));
    } else {
        panic!("Expected unary NOT!");
    }
}

fn split_skip_parens<F>(items: &[Item], delim: Item, f: F) -> bool where F: FnMut(&[Item]) {
    let mut prev_idx = 0;
    let mut layer = 0;

    for (idx, item) in items.iter().enumerate() {
        use Item::*;
        match item {
            LeftParens => layer += 1;
            RightParens => {
                assert!(layer > 0, "Mismatched parentheses!");
                layer -= 1;
            }
            _ if layer == 0 && item == delim => {
                f(items[prev_idx..idx]);
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
    let c1 = items.iter().take_while(|i| i == Item::LeftParens).count();
    let c2 = items.iter().rev().take_while(|i| i == Item::RightParens).count();
    let c = c1.min(c2);
    &items[c..items.len() - c]
}
