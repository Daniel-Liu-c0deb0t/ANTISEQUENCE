use crate::iter::*;

pub struct SetReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    label_or_attr: LabelOrAttr,
    format_expr: FormatExpr,
}

impl<R: Reads> SetReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        label_or_attr: LabelOrAttr,
        format_expr: FormatExpr,
    ) -> Self {
        Self {
            reads,
            selector_expr,
            label_or_attr,
            format_expr,
        }
    }
}

impl<R: Reads> Reads for SetReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if self.selector_expr.matches(&read).map_err(|e| Error::NameError { source: e, read: read.clone(), context: "setting reads" })? {
                continue;
            }

            let new_str = self.format_expr.format(read, false)
                .map_err(|e| Error::NameError { source: e, read: read.clone(), context: "setting reads" })?;

            match &self.label_or_attr {
                LabelOrAttr::Label(label) => {
                    if read.str_mappings(label.str_type).unwrap().qual().is_some() {
                        let new_qual = self.format_expr.format(read, true);
                        read.set(label.str_type, label.label, &new_str, Some(&new_qual));
                    } else {
                        read.set(label.str_type, label.label, &new_str, None);
                    }
                }
                LabelOrAttr::Attr(attr) => {
                    *read.
                        data_mut(attr.str_type, attr.label, attr.attr)
                        .map_err(|e| Error::NameError { source: e, read: read.clone(), context: "setting reads" }) = Data::Bytes(new_str);
                }
            }
        }

        Ok(reads)
    }

    fn finish(&self) -> Result<()> {
        self.reads.finish()
    }
}
