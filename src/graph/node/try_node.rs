use crate::graph::*;

pub struct TryNode {
    try_graph: Graph,
    catch_graph: Graph,
}

impl TryNode {
    const NAME: &'static str = "TryNode";

    /// Run reads through the try graph, remove the ones that have skipped an operation,
    /// and then run the skipped reads through the catch graph.
    ///
    /// An operation is skipped only if the read does not have a name (label or attribute)
    /// that is required by the operation.
    /// This is useful for specifying a chain of operations where each operation depends on the
    /// labels or attributes produced by the previous operation.
    pub fn new(try_graph: Graph, catch_graph: Graph) -> Self {
        Self {
            try_graph,
            catch_graph,
        }
    }
}

impl GraphNode for TryNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let (read, failed, done) = self.try_graph.try_run_one(read)?;

        if !done && read.is_some() && failed {
            let (_, done) = self.catch_graph.run_one(read)?;
            Ok((None, done))
        } else {
            Ok((read, done))
        }
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
