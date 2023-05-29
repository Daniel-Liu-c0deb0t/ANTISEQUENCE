use crate::iter::*;

pub struct TrimReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    labels: Vec<Label>,
}

impl<R: Reads> TrimReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr, labels: Vec<Label>) -> Self {
        Self {
            reads,
            selector_expr,
            labels,
        }
    }
}

impl<R: Reads> Reads for TrimReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "trim reads",
                })?)
            {
                continue;
            }

            self.labels
                .iter()
                .try_for_each(|l| read.trim(l.str_type, l.label))
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "trim reads",
                })?;
        }

        Ok(reads)
    }

    fn finish(self) -> Result<()> {
        self.reads.finish()
    }
}
