use crate::graph::*;

pub struct ForEachNode<F: Fn(&mut Read) + Send + Sync> {
    func: F,
}

impl<F: Fn(&mut Read) + Send + Sync> ForEachNode<F> {
    const NAME: &'static str = "for each read";

    pub fn new(func: F) -> Self {
        Self {
            func,
        }
    }
}

impl<F: Fn(&mut Read) + Send + Sync> GraphNode for ForEachNode<F> {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };
        (self.func)(&mut read);
        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}

pub struct DbgNode;

impl DbgNode {
    pub fn new() -> ForEachNode<impl Fn(&mut Read) + Send + Sync> {
        ForEachNode::new(|read| eprintln!("{read}"))
    }
}

pub struct RemoveInternalNode;

impl RemoveInternalNode {
    pub fn new() -> ForEachNode<impl Fn(&mut Read) + Send + Sync> {
        ForEachNode::new(|read| read.remove_internal())
    }
}