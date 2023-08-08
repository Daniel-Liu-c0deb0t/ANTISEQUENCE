use crate::iter::*;

pub struct ForkNode {
    graph: Graph,
}

impl ForkNode {
    const NAME: &'static str = "forking reads";

    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }
}

impl GraphNode for ForkNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(read) = read else { panic!("Expected some read!") };
        self.graph.run_one(Some(read.clone()));
        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
