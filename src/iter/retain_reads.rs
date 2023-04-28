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
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();
        reads.retain(|r| self.selector_expr.matches(r));
        reads
    }
}
