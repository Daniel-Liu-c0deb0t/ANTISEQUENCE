use crate::iter::*;

pub struct CutReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    cut_label: Label,
    new_label1: Option<Label>,
    new_label2: Option<Label>,
    cut_idx: EndIdx,
}

impl<R: Reads> CutReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        cut_idx: EndIdx,
    ) -> Self {
        transform_expr.check_size(1, 2, "cutting reads");
        transform_expr.check_same_str_type("cutting reads");

        Self {
            reads,
            selector_expr,
            cut_label: transform_expr.before()[0].clone(),
            new_label1: transform_expr.after()[0].clone().map(|l| match l {
                LabelOrAttr::Label(l) => l,
                _ => panic!("Expected type.label after the \"->\" in the transform expression when cutting reads"),
            }),
            new_label2: transform_expr.after()[1].clone().map(|l| match l {
                LabelOrAttr::Label(l) => l,
                _ => panic!("Expected type.label after the \"->\" in the transform expression when cutting reads"),
            }),
            cut_idx,
        }
    }
}

impl<R: Reads> Reads for CutReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "cutting reads",
                })?)
            {
                continue;
            }

            read.cut(
                self.cut_label.str_type,
                self.cut_label.label,
                self.new_label1.as_ref().map(|l| l.label),
                self.new_label2.as_ref().map(|l| l.label),
                self.cut_idx,
            )
            .map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: "cutting read",
            })?;
        }

        Ok(reads)
    }

    fn finish(self) -> Result<()> {
        self.reads.finish()
    }
}
