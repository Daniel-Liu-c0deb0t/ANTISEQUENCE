use crate::graph::*;

pub struct RetainNode {
    required_names: Vec<LabelOrAttr>,
    selector_expr: Expr,
}

impl RetainNode {
    const NAME: &'static str = "RetainNode";

    /// Retain only the reads where the selector expression evaluates to true and discard the rest.
    pub fn new(selector_expr: Expr) -> Self {
        Self {
            required_names: selector_expr.required_names(),
            selector_expr,
        }
    }
}

impl GraphNode for RetainNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(read) = read else { panic!("Expected some read!") };

        if self.selector_expr.eval_bool(&read).map_err(|e| Error::NameError {
            source: e,
            read: read.clone(),
            context: Self::NAME,
        })? {
            Ok((Some(read), false))
        } else {
            Ok((None, false))
        }
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &self.required_names
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
