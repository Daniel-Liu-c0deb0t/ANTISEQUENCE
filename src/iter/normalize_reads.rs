use std::ops::{Bound, Div};

use crate::iter::*;

pub struct NormalizeReads<R: Reads, B: RangeBounds<usize> + Send + Sync> {
    reads: R,
    selector_expr: SelectorExpr,
    label: Label,
    range: B,
}

pub const NUC_MAP: [u8; 4] = [b'A', b'C', b'T', b'G'];

pub fn log4_roundup(n: usize) -> usize {
    ((usize::BITS - n.leading_zeros()) as f64).div(2.0).ceil() as usize
}

impl<R: Reads, B: RangeBounds<usize> + Send + Sync> NormalizeReads<R, B> {
    pub fn new(reads: R, selector_expr: SelectorExpr, label: Label, range: B) -> Self {
        Self {
            reads,
            selector_expr,
            label,
            range,
        }
    }
}

impl<R: Reads, B: RangeBounds<usize> + Send + Sync> Reads for NormalizeReads<R, B> {
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

            let a = if let Bound::Included(a) = self.range.start_bound() {
                *a
            } else {
                return Err(Error::NameError {
                    source: NameError::TypeExplicit("inclusive start bound", "unbounded start"),
                    read: read.clone(),
                    context: "normalizing read",
                });
            };

            let b = match self.range.end_bound() {
                Bound::Excluded(b) => *b - 1,
                Bound::Included(b) => *b,
                Bound::Unbounded => {
                    return Err(Error::NameError {
                        source: NameError::TypeExplicit(
                            "inclusive or exclusive start bound",
                            "unbounded start",
                        ),
                        read: read.clone(),
                        context: "normalizing read",
                    })
                }
            };

            read.norm(self.label.str_type, self.label.label, a, b)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "normalizing read",
                })?;
        }

        Ok(reads)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()
    }
}
