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
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            self.labels
                .iter()
                .for_each(|l| read.trim(l.str_type, &l.label));
        }

        reads
    }
}
