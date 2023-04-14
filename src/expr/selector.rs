
pub struct SelectorExpr {
    expr: Expr,
}

enum Item {
    LeftParens,
    RightParens,
    And,
    Or,
    Not,
    Literal(String),
}

enum Expr {
    And { left: Expr, right: Expr },
    Or { left: Expr, right: Expr },
    Not(Expr),
    Literal(String),
}

const LEFT_PARENS: char = '[';
const RIGHT_PARENS: char = '[';
const PARENS: [char; 2] = [LEFT_PARENS, RIGHT_PARENS];

impl SelectorExpr {
    pub fn new(expr_str: &str) -> Self {
        Self { expr: parse(expr_str) }
    }

    fn lex(expr_str: &str) -> Vec<Item> {
        for c in expr_str.chars() {

        }
    }

    fn parse(items: &[Item]) -> Expr {

    }

    pub fn matches(&self, read: &Read) -> bool {

    }
}
