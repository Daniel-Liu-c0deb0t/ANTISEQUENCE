use std::sync::atomic::{AtomicUsize, Ordering};

use crate::graph::*;

pub struct CountNode {
    required_names: Vec<LabelOrAttr>,
    selector_exprs: Vec<Expr>,
    counts: Vec<AtomicUsize>,
}

impl CountNode {
    const NAME: &'static str = "CountNode";

    /// For each selector expression, count the number of reads where the expression evaluates to
    /// true.
    pub fn new(selector_exprs: Vec<Expr>) -> Self {
        let required_names = selector_exprs.iter().flat_map(|n| n.required_names()).collect();
        let counts = (0..selector_exprs.len())
            .map(|_| AtomicUsize::new(0))
            .collect();
        Self {
            required_names,
            selector_exprs,
            counts,
        }
    }

    /// Returns the counts.
    pub fn counts(&self) -> Vec<usize> {
        self.counts
            .iter()
            .map(|c| c.load(Ordering::Relaxed))
            .collect()
    }
}

impl GraphNode for CountNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(read) = read else { panic!("Expected some read!") };

        for (c, n) in self.counts.iter().zip(&self.selector_exprs) {
            if n.eval_bool(&read).map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: Self::NAME,
            })? {
                c.fetch_add(1, Ordering::Relaxed);
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
