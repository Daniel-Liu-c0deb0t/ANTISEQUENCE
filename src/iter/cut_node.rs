use crate::iter::*;

pub struct CutNode {
    next_node: Option<Box<dyn GraphNode>>,
    required_names: Vec<LabelOrAttr>,

    cut_label: Label,
    new_label1: Option<Label>,
    new_label2: Option<Label>,
    cut_idx: EndIdx,
}

impl CutNode {
    const NAME: &'static str = "cutting reads";

    pub fn new(
        transform_expr: TransformExpr,
        cut_idx: EndIdx,
    ) -> Self {
        transform_expr.check_size(1, 2, Self::NAME);
        transform_expr.check_same_str_type(Self::NAME);

        Self {
            next_node: None,
            required_names: vec![transform_expr.before(0).into()],
            cut_label: transform_expr.before(0),
            new_label1: transform_expr.after_label(0, Self::NAME),
            new_label2: transform_expr.after_label(1, Self::NAME),
            cut_idx,
        }
    }
}

impl GraphNode for CutNode {
    fn run(&self, read: Option<Read>, next_nodes: &mut Vec<&dyn GraphNode>) -> Result<(Option<Read>, bool)> {
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

        if let Some(node) = &self.next_node {
            next_nodes.push(&**node);
        }
        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &self.required_names
    }

    fn cond(&self) -> Option<Node> {
        None
    }

    fn set_next(&mut self, node: Box<dyn GraphNode>) -> &mut dyn GraphNode {
        self.next_node = Some(node);
        &mut **self.next_node.as_mut().unwrap()
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
