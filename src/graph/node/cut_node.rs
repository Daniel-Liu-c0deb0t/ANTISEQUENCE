use crate::graph::*;

pub struct CutNode {
    required_names: Vec<LabelOrAttr>,
    cut_label: Label,
    new_label1: Option<Label>,
    new_label2: Option<Label>,
    cut_idx: EndIdx,
}

impl CutNode {
    const NAME: &'static str = "CutNode";

    /// Cut a labeled interval at the specified index to create two new intervals.
    ///
    /// The transform expression must have one input label and two output labels.
    ///
    /// Example `transform_expr`: `tr!(seq1.* -> seq1.left, seq1.right)`.
    pub fn new(
        transform_expr: TransformExpr,
        cut_idx: EndIdx,
    ) -> Self {
        transform_expr.check_size(1, 2, Self::NAME);
        transform_expr.check_same_str_type(Self::NAME);

        Self {
            required_names: vec![transform_expr.before(0).into()],
            cut_label: transform_expr.before(0),
            new_label1: transform_expr.after_label(0, Self::NAME),
            new_label2: transform_expr.after_label(1, Self::NAME),
            cut_idx,
        }
    }
}

impl GraphNode for CutNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

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
            context: self.name(),
        })?;

        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &self.required_names
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
