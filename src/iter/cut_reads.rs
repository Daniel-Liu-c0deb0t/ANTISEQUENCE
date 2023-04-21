use crate::iter::*;

pub struct CutReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    transform_expr: TransformExpr,
    cut_idx: EndIdx,
}

impl<R: Reads> CutReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        cut_idx: EndIdx,
    ) -> Self {
        transform_expr.check(1, 2);

        Self {
            reads,
            selector_expr,
            transform_expr,
            cut_idx,
        }
    }
}

impl<R: Reads> Reads for CutReads<R> {
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            read.cut(
                self.transform_expr.before()[0].str_type,
                &self.transform_expr.before()[0].label,
                &self.transform_expr.after()[0].label,
                &self.transform_expr.after()[1].label,
                self.cut_idx,
            );
        }

        reads
    }
}
