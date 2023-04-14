use crate::read::*;

pub struct CollectFastqReads<'r, R: Reads> {
    reads: &'r R,
    selector_expr: SelectorExpr,
    file_fmt: FormatExpr,
}

impl<'r, R: Reads> CollectFastqReads<'r, R> {
    pub fn new(reads: &'r R, selector_expr: SelectorExpr, file_expr: FormatExpr) -> Self {
        Self {
            reads,
            selector_expr,
            file_expr,
        }
    }
}

impl<'r, R: Reads> Reads for CollectFastqReads<'r, R> {
    fn next_chunk() -> Read {
        let mut reads = self.reads.next_chunk();

        for read in &mut reads {

        }

        reads
    }
}
