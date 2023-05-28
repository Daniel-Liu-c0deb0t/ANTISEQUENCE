use std::ops::RangeBounds;

use crate::iter::*;

pub struct LengthInBoundsReads<R: Reads, B: RangeBounds<usize> + Send + Sync> {
    reads: R,
    selector_expr: SelectorExpr,
    label: Label,
    attr: Option<Attr>,
    bounds: B,
}

impl<R: Reads, B: RangeBounds<usize> + Send + Sync> LengthInBoundsReads<R, B> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        bounds: B,
    ) -> Self {
        transform_expr.check_size(1, 1, "checking length in bounds");
        transform_expr.check_same_str_type("checking length in bounds");

        Self {
            reads,
            selector_expr,
            label: transform_expr.before()[0].clone(),
            attr: transform_expr.after()[0].clone().map(|a| match a {
                LabelOrAttr::Attr(a) => a,
                _ => panic!("Expected type.label.attr after the \"->\" in the transform expression when checking length in bounds"),
            }),
            bounds,
        }
    }
}

impl<R: Reads, B: RangeBounds<usize> + Send + Sync> Reads for LengthInBoundsReads<R, B> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "checking length in bounds",
                })?)
            {
                continue;
            }

            if let Some(attr) = &self.attr {
                let len = read
                    .mapping(self.label.str_type, self.label.label)
                    .map_err(|e| Error::NameError {
                        source: e,
                        read: read.clone(),
                        context: "checking length in bounds",
                    })?
                    .len;

                // panic to make borrow checker happy
                *read
                    .data_mut(attr.str_type, attr.label, attr.attr)
                    .unwrap_or_else(|e| panic!("Error checking length in bounds: {e}")) =
                    Data::Bool(self.bounds.contains(&len));
            }
        }

        Ok(reads)
    }

    fn finish(&self) -> Result<()> {
        self.reads.finish()
    }
}
