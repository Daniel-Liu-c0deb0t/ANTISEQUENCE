use crate::iter::*;

pub struct PadReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    labels: Vec<Label>,
    max_length: EndIdx,
    pad_char: u8,
}

impl<R: Reads> PadReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        labels: Vec<Label>,
        max_length: EndIdx,
        pad_char: u8,
    ) -> Self {
        Self {
            reads,
            selector_expr,
            labels,
            max_length,
            pad_char,
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
                .try_for_each(|l| read.pad(l.str_type, l.label, self.max_length, self.pad_char))
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
