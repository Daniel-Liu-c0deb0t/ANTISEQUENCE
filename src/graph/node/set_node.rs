use crate::graph::*;

pub struct SetNode {
    required_names: Vec<LabelOrAttr>,
    label_or_attr: LabelOrAttr,
    expr: Expr,
}

impl SetNode {
    const NAME: &'static str = "setting reads";

    pub fn new(
        label_or_attr: LabelOrAttr,
        expr: Expr,
    ) -> Self {
        Self {
            required_names: vec![label_or_attr.clone()],
            label_or_attr,
            expr,
        }
    }
}

impl GraphNode for SetNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

        match &self.label_or_attr {
            LabelOrAttr::Label(label) => {
                let new_bytes = self.expr
                    .eval_bytes(&read, false)
                    .map_err(|e| Error::NameError {
                        source: e,
                        read: read.clone(),
                        context: Self::NAME,
                    })?;

                let str_mappings =
                    read.str_mappings(label.str_type)
                        .ok_or_else(|| Error::NameError {
                            source: NameError::NotInRead(Name::StrType(label.str_type)),
                            read: read.clone(),
                            context: Self::NAME,
                        })?;

                if str_mappings.qual().is_some() {
                    let new_qual =
                        self.expr
                            .eval_bytes(&read, true)
                            .map_err(|e| Error::NameError {
                                source: e,
                                read: read.clone(),
                                context: Self::NAME,
                            })?;

                    read.set(label.str_type, label.label, &new_bytes, Some(&new_qual))
                        .map_err(|e| Error::NameError {
                            source: e,
                            read: read.clone(),
                            context: Self::NAME,
                        })?;
                } else {
                    read.set(label.str_type, label.label, &new_bytes, None)
                        .map_err(|e| Error::NameError {
                            source: e,
                            read: read.clone(),
                            context: Self::NAME,
                        })?;
                }
            }
            LabelOrAttr::Attr(attr) => {
                let new_val = self.expr
                    .eval(&read, false)
                    .map_err(|e| Error::NameError {
                        source: e,
                        read: read.clone(),
                        context: Self::NAME,
                    })?;

                // panic to make borrow checker happy
                *read
                    .data_mut(attr.str_type, attr.label, attr.attr)
                    .unwrap_or_else(|e| panic!("Error {}: {e}", Self::NAME)) = new_val;
            }
        }

        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &self.required_names
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
