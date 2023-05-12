use crate::iter::*;

pub struct ForEachReads<R: Reads, F: Fn(&mut Read) + std::marker::Sync> {
    reads: R,
    selector_expr: SelectorExpr,
    func: F,
}

impl<R: Reads, F: Fn(&mut Read) + std::marker::Sync> ForEachReads<R, F> {
    pub fn new(reads: R, selector_expr: SelectorExpr, func: F) -> Self {
        Self {
            reads,
            selector_expr,
            func,
        }
    }
}

impl<R: Reads, F: Fn(&mut Read) + std::marker::Sync> Reads for ForEachReads<R, F> {
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();
        reads
            .iter_mut()
            .filter(|r| self.selector_expr.matches(r))
            .for_each(|read| (self.func)(read));
        reads
    }

    fn finish(&self) {
        self.reads.finish();
    }
}
