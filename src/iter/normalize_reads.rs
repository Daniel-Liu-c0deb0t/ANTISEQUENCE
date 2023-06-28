use crate::iter::*;

pub struct NormalizeReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    label: Label,
    range: (usize, usize),
}

pub const NUC_MAP: [u8; 4] = [b'A', b'C', b'T', b'G'];

pub fn log2_roundup(n: usize) -> usize {
    let mut log = 1;

    let mut n = n >> 2;

    while n != 0 {
        n >>= 1;
        log += 1
    }

    log
}

impl<R: Reads> NormalizeReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr, label: Label, range: (usize, usize)) -> Self {
        Self {
            reads,
            selector_expr,
            label,
            range,
        }
    }
}

impl<R: Reads> Reads for NormalizeReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "normalize reads",
                })?)
            {
                continue;
            }

            read.norm(self.label.str_type, self.label.label, self.range)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "normalizing reads",
                })?;
        }

        Ok(reads)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()
    }
}
