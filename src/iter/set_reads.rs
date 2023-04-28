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
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            let new_str = self.format_expr.format(read, false);

            match &self.label_or_attr {
                LabelOrAttr::Label(label) => {
                    if read.str_mappings(label.str_type).unwrap().qual().is_some() {
                        let new_qual = self.format_expr.format(read, true);
                        read.set(
                            label.str_type,
                            label.label,
                            new_str.as_bytes(),
                            Some(new_qual.as_bytes()),
                        );
                    } else {
                        read.set(label.str_type, label.label, new_str.as_bytes(), None);
                    }
                }
                LabelOrAttr::Attr(attr) => {
                    *read
                        .str_mappings_mut(attr.str_type)
                        .unwrap()
                        .mapping_mut(attr.label)
                        .unwrap()
                        .data_mut(attr.attr) = Data::String(new_str)
                }
            }
        }

        reads
    }
}
