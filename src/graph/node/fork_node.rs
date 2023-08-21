use crate::graph::*;

pub struct ForkNode {
    graph: Graph,
}

impl ForkNode {
    const NAME: &'static str = "ForkNode";

    /// Clone each read and run the clone through the specified graph, while leaving
    /// the original read unchanged.
    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }
}

impl GraphNode for ForkNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(read) = read else { panic!("Expected some read!") };
        self.graph.run_one(Some(read.clone()))?;
        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
