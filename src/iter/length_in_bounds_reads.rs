use std::ops::RangeBounds;

use crate::iter::*;

pub struct LengthInBoundsReads<R: Reads, B: RangeBounds<usize> + std::marker::Sync> {
    reads: R,
    selector_expr: SelectorExpr,
    transform_expr: TransformExpr,
    bounds: B,
}

impl<R: Reads, B: RangeBounds<usize> + std::marker::Sync> LengthInBoundsReads<R, B> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        bounds: B,
    ) -> Self {
        transform_expr.check_size(1, 1);
        transform_expr.check_same_str_type();

        Self {
            reads,
            selector_expr,
            transform_expr,
            bounds,
        }
    }
}

impl<R: Reads, B: RangeBounds<usize> + std::marker::Sync> Reads for LengthInBoundsReads<R, B> {
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            if let Some(label_or_attr) = self.transform_expr.after()[0].as_ref() {
                let LabelOrAttr::Attr(after) = label_or_attr else {
                    panic!("Expected type.label.attr!")
                };

                let before = &self.transform_expr.before()[0];
                let len = read
                    .str_mappings(before.str_type)
                    .unwrap()
                    .mapping(before.label)
                    .unwrap()
                    .len;

                *read
                    .str_mappings_mut(after.str_type)
                    .unwrap()
                    .mapping_mut(after.label)
                    .unwrap()
                    .data_mut(after.attr) = Data::Bool(self.bounds.contains(&len));
            }
        }

        reads
    }

    fn finish(&self) {
        self.reads.finish();
    }
}
