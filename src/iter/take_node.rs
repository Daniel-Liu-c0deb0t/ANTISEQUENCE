use crate::iter::*;

pub struct TakeNode<B: RangeBounds<usize> + Send + Sync> {
    bounds: B,
}

impl<B: RangeBounds<usize> + Send + Sync> TakeNode<B> {
    const NAME: &'static str = "taking reads";

    pub fn new(bounds: B) -> Self {
        Self { bounds }
    }
}

impl<B: RangeBounds<usize> + Send + Sync> GraphNode for TakeNode<B> {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(read) = read else { panic!("Expected some read!") };
        let first_idx = read.first_idx();

        if self.bounds.contains(&first_idx) {
            Ok((Some(read), false))
        } else {
            let done = match self.bounds.upper_bound() {
                Include(hi) => first_idx > hi,
                Exclude(hi) => first_idx >= hi,
                Unbounded => false,
            };
            Ok((None, done))
        }
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &self.required_names
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
