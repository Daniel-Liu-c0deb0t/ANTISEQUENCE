use std::ops::RangeBounds;

use crate::iter::*;

pub struct LengthInBoundsReads<R: Reads, B: RangeBounds<usize> + std::marker::Sync> {
    reads: R,
    selector_expr: SelectorExpr,
    attr: Attr,
    bounds: B,
}

impl<R: Reads, B: RangeBounds<usize> + std::marker::Sync> LengthInBoundsReads<R, B> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        attr: Attr,
        bounds: B,
    ) -> Self {
        Self {
            reads,
            selector_expr,
            attr,
            bounds,
        }
    }
}

impl<R: Reads, B: RangeBounds<usize> + std::marker::Sync> Reads for LengthInBoundsReads<R, B> {
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            let mapping = read.get_str_mappings_mut(self.attr.str_type).unwrap()
            .get_mapping_mut(self.attr.label).unwrap();
            *mapping.get_data_mut(self.attr.attr) = Data::Bool(self.bounds.contains(&mapping.len));
        }

        reads
    }
}
