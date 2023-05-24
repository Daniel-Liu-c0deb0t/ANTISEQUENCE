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
            if self
                .selector_expr
                .matches(&read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "setting reads",
                })?
            {
                continue;
            }

            let new_str = self
                .format_expr
                .format(read, false)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "setting reads",
                })?;

            match &self.label_or_attr {
                LabelOrAttr::Label(label) => {
                    let str_mappings =
                        read.str_mappings(label.str_type)
                            .ok_or_else(|| Error::NameError {
                                source: NameError::NotInRead(Name::StrType(label.str_type)),
                                read: read.clone(),
                                context: "setting reads",
                            })?;

                    if str_mappings.qual().is_some() {
                        let new_qual =
                            self.format_expr
                                .format(read, true)
                                .map_err(|e| Error::NameError {
                                    source: e,
                                    read: read.clone(),
                                    context: "setting reads",
                                })?;
                        read.set(label.str_type, label.label, &new_str, Some(&new_qual))
                            .map_err(|e| Error::NameError {
                                source: e,
                                read: read.clone(),
                                context: "setting reads",
                            })?;
                    } else {
                        read.set(label.str_type, label.label, &new_str, None)
                            .map_err(|e| Error::NameError {
                                source: e,
                                read: read.clone(),
                                context: "setting reads",
                            })?;
                    }
                }
                LabelOrAttr::Attr(attr) => {
                    // panic to make borrow checker happy
                    *read
                        .data_mut(attr.str_type, attr.label, attr.attr)
                        .unwrap_or_else(|e| panic!("Error setting reads: {e}")) =
                        Data::Bytes(new_str);
                }
            }
        }

        Ok(reads)
    }

    fn finish(&self) -> Result<()> {
        self.reads.finish()
    }
}
