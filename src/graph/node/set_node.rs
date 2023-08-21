use crate::graph::*;

pub struct SetNode {
    required_names: Vec<LabelOrAttr>,
    label_or_attr: LabelOrAttr,
    expr: Expr,
}

impl SetNode {
    const NAME: &'static str = "SetNode";

    /// Set a labeled interval or attribute to the result of an expression.
    ///
    /// The expression must return a byte string if a labeled interval is being set.
    ///
    /// To generate the quality scores when setting intervals that have corresponding quality
    /// scores, references to intervals in the expression are directly substituted with the
    /// corresponding quality scores of the intervals. For references to byte strings without
    /// quality scores, a sequence of `I`s is used as the quality scores in the expression.
    /// *This naive substitution may lead to unexpected results for complex expressions!*
    ///
    /// If a label is set, then its interval and all other intersecting intervals will be adjusted accordingly
    /// for any shortening or lengthening.
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
                    })?.into_owned();

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
                            })?.into_owned();

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
                    .unwrap_or_else(|e| panic!("Error in {}: {e}", Self::NAME)) = new_val.to_data();
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
