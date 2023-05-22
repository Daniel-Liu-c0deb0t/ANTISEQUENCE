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

impl<R: Reads> Reads for RetainReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let reads = self.reads.next_chunk()?;
        let mut res = Vec::new();
        for read in reads.into_iter() {
            if self.selector_expr.matches(&read).map_err(|e| Error::NameError { source: e, read: read.clone(), context: "retain reads" })? {
                res.push(read);
            }
        }
        Ok(res)
    }

    fn finish(&self) -> Result<()> {
        self.reads.finish()
    }
}
