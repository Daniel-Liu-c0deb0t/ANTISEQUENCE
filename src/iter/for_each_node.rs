use crate::iter::*;

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

impl GraphNode for ForEachNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(read) = read else { panic!("Expected some read!") };
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
