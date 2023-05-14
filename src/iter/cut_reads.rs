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
        transform_expr.check_size(1, 2);
        transform_expr.check_same_str_type();

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
                self.transform_expr.before()[0].label,
                self.transform_expr.after()[0].as_ref().map(|l| match l {
                    LabelOrAttr::Label(l) => l.label,
                    _ => panic!("Expected type.label!"),
                }),
                self.transform_expr.after()[1].as_ref().map(|l| match l {
                    LabelOrAttr::Label(l) => l.label,
                    _ => panic!("Expected type.label!"),
                }),
                self.cut_idx,
            );
        }

        reads
    }

    fn finish(&self) {
        self.reads.finish();
    }
}
