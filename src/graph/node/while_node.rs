use crate::graph::*;

pub struct WhileNode {
    required_names: Vec<LabelOrAttr>,
    cond_expr: Expr,
    graph: Graph,
}

impl WhileNode {
    const NAME: &'static str = "WhileNode";

    /// Run a read through the graph multiple times, while the condition expression evaluates to true.
    pub fn new(cond_expr: Expr, graph: Graph) -> Self {
        let required_names = cond_expr.required_names();
        Self {
            required_names,
            cond_expr,
            graph,
        }
    }
}

impl GraphNode for WhileNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

        while self.cond_expr.eval_bool(&read).map_err(|e| Error::NameError {
            source: e,
            read: read.clone(),
            context: Self::NAME,
        })? {
            let (r, done) = self.graph.run_one(Some(read))?;

            if done {
                return Ok((r, done));
            }

            if let Some(r) = r {
                read = r;
            } else {
                return Ok((r, done));
            }
        }

        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &self.required_names
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
