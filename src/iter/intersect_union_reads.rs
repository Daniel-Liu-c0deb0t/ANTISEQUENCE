use crate::iter::*;

pub struct IntersectReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    label1: Label,
    label2: Label,
    new_label: Option<Label>,
}

impl<R: Reads> IntersectReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr, transform_expr: TransformExpr) -> Self {
        transform_expr.check_size(2, 1, "intersecting mappings in reads");
        transform_expr.check_same_str_type("intersecting mappings in reads");

        Self {
            reads,
            selector_expr,
            label1: transform_expr.before()[0].clone(),
            label2: transform_expr.before()[1].clone(),
            new_label: transform_expr.after()[0].clone().map(|l| match l {
                LabelOrAttr::Label(l) => l,
                _ => panic!("Expected type.label after the \"->\" in the transform expression when intersecting mappings in reads"),
            }),
        }
    }
}

impl<R: Reads> Reads for IntersectReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "intersecting mappings in reads",
                })?)
            {
                continue;
            }

            read.intersect(
                self.label1.str_type,
                self.label1.label,
                self.label2.label,
                self.new_label.as_ref().map(|l| l.label),
            )
            .map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: "intersecting mappings in reads",
            })?;
        }

        Ok(reads)
    }

    fn finish(self) -> Result<()> {
        self.reads.finish()
    }
}

pub struct UnionReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    label1: Label,
    label2: Label,
    new_label: Option<Label>,
}

impl<R: Reads> UnionReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr, transform_expr: TransformExpr) -> Self {
        transform_expr.check_size(2, 1, "unioning mappings in reads");
        transform_expr.check_same_str_type("unioning mappings in reads");

        Self {
            reads,
            selector_expr,
            label1: transform_expr.before()[0].clone(),
            label2: transform_expr.before()[1].clone(),
            new_label: transform_expr.after()[0].clone().map(|l| match l {
                LabelOrAttr::Label(l) => l,
                _ => panic!("Expected type.label after the \"->\" in the transform expression when unioning mappings in reads"),
            }),
        }
    }
}

impl<R: Reads> Reads for UnionReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "unioning mappings in reads",
                })?)
            {
                continue;
            }

            read.union(
                self.label1.str_type,
                self.label1.label,
                self.label2.label,
                self.new_label.as_ref().map(|l| l.label),
            )
            .map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: "unioning mappings in reads",
            })?;
        }

        Ok(reads)
    }

    fn finish(self) -> Result<()> {
        self.reads.finish()
    }
}
