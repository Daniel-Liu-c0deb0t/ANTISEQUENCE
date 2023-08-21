use crate::graph::*;

pub struct SelectNode {
    required_names: Vec<LabelOrAttr>,
    selector_expr: Expr,
    graph: Graph,
}

impl SelectNode {
    const NAME: &'static str = "SelectNode";

    /// Run the graph only on reads where the selector expression evaluates to true.
    pub fn new(selector_expr: Expr, graph: Graph) -> Self {
        let required_names = selector_expr.required_names();
        Self {
            required_names,
            selector_expr,
            graph,
        }
    }
}

impl GraphNode for SelectNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(read) = read else { panic!("Expected some read!") };

        if self.selector_expr.eval_bool(&read).map_err(|e| Error::NameError {
            source: e,
            read: read.clone(),
            context: Self::NAME,
        })? {
            self.graph.run_one(Some(read))
        } else {
            Ok((Some(read), false))
        }
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &self.required_names
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
