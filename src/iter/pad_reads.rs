use crate::iter::*;

pub struct PadReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    labels: Vec<Label>,
    to_length: usize,
}

pub const VAR_LEN_BC_PADDING: &[&str] = &["A", "CA", "GAA", "TAAA"];

impl<R: Reads> PadReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        labels: Vec<Label>,
        to_length: usize,
    ) -> Self {
        Self {
            reads,
            selector_expr,
            labels,
            to_length,
        }
    }
}

impl<R: Reads> Reads for PadReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "pad reads",
                })?)
            {
                continue;
            }

            self.labels
                .iter()
                .try_for_each(|l| read.pad(l.str_type, l.label, self.to_length))
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "pad reads",
                })?;
        }

        Ok(reads)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()
    }
}
