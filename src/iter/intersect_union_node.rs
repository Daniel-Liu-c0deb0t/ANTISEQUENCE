use crate::iter::*;

pub struct IntersectNode {
    required_names: Vec<LabelOrAttr>,
    label1: Label,
    label2: Label,
    new_label: Option<Label>,
}

impl IntersectNode {
    const NAME: &'static str = "intersecting intervals in reads";

    pub fn new(transform_expr: TransformExpr) -> Self {
        transform_expr.check_size(2, 1, Self::NAME);
        transform_expr.check_same_str_type(Self::NAME);

        Self {
            required_names: vec![transform_expr.before(0).into(), transform_expr.before(1).into()],
            label1: transform_expr.before(0),
            label2: transform_expr.before(1),
            new_label: transform_expr.after_label(0, Self::NAME),
        }
    }
}

impl GraphNode for IntersectNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

        read.intersect(
            self.label1.str_type,
            self.label1.label,
            self.label2.label,
            self.new_label.as_ref().map(|l| l.label),
        )
        .map_err(|e| Error::NameError {
            source: e,
            read: read.clone(),
            context: Self::NAME,
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

pub struct UnionNode {
    required_names: Vec<LabelOrAttr>,
    label1: Label,
    label2: Label,
    new_label: Option<Label>,
}

impl UnionNode {
    const NAME: &'static str = "unioning intervals in reads";

    pub fn new(transform_expr: TransformExpr) -> Self {
        transform_expr.check_size(2, 1, Self::NAME);
        transform_expr.check_same_str_type(Self::NAME);

        Self {
            required_names: vec![transform_expr.before(0).into(), transform_expr.before(1).into()],
            label1: transform_expr.before(0),
            label2: transform_expr.before(1),
            new_label: transform_expr.after_label(0, Self::NAME),
        }
    }
}

impl GraphNode for UnionNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

        read.union(
            self.label1.str_type,
            self.label1.label,
            self.label2.label,
            self.new_label.as_ref().map(|l| l.label),
        )
        .map_err(|e| Error::NameError {
            source: e,
            read: read.clone(),
            context: Self::NAME,
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
