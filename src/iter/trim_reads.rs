use crate::iter::*;

pub struct TrimReads<'r, R: Reads> {
    reads: &'r R,
    selector_expr: SelectorExpr,
    labels: Vec<Label>,
}

impl<'r, R: Reads> TrimReads<'r, R> {
    pub fn new(reads: &'r R, selector_expr: SelectorExpr, labels: Vec<Label>) -> Self {
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

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            self.labels.iter().for_each(|l| read.trim(l.str_type, &l.label));
        }

        reads
    }
}
