use crate::iter::*;

pub struct RetainReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
}

impl<R: Reads> RetainReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr) -> Self {
        Self {
            reads,
            selector_expr,
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

impl<R: Reads> Reads for RetainReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let reads = self.reads.next_chunk()?;
        let mut res = Vec::new();
        for read in reads.into_iter() {
            if self
                .selector_expr
                .matches(&read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "retain reads",
                })?
            {
                res.push(read);
            }
        }
        Ok(res)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()
    }
}
