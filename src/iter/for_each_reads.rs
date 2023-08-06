use crate::iter::*;

pub struct ForEachReads<R: Reads, F: Fn(&mut Read) + Send + Sync> {
    reads: R,
    selector_expr: SelectorExpr,
    func: F,
}

impl<R: Reads, F: Fn(&mut Read) + Send + Sync> ForEachReads<R, F> {
    pub fn new(reads: R, selector_expr: SelectorExpr, func: F) -> Self {
        Self {
            reads,
            selector_expr,
            func,
        }
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

impl<R: Reads, F: Fn(&mut Read) + Send + Sync> Reads for ForEachReads<R, F> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;
        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "for each",
                })?)
            {
                continue;
            }

            (self.func)(read);
        }
        Ok(reads)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()
    }
}
