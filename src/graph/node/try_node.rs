use crate::graph::*;

pub struct TryNode {
    try_graph: Graph,
    catch_graph: Graph,
}

impl TryNode {
    const NAME: &'static str = "trying operations on reads";

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
            self.catch_graph.run_one(read)
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
