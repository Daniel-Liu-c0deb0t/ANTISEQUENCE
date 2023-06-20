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
