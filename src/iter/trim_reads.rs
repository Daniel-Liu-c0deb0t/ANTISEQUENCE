use crate::iter::*;
use crate::read::*;

pub struct TrimReads<'r, R: Reads> {
    reads: &'r R,
    selector_expr: SelectorExpr,
    labels: Vec<String>,
}

impl<'r, R: Reads> TrimReads<'r, R> {
    pub fn new(reads: &'r R, selector_expr: SelectorExpr, labels: String) -> Self {
        Self {
            reads,
            selector_expr,
            labels,
        }
    }
}

impl<'r, R: Reads> Reads for TrimReads<'r, R> {
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();

        for read in &mut reads {
            self.labels.iter().for_each(|l| read.trim(l));
        }

        reads
    }
}
