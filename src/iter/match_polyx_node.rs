use crate::iter::*;

pub struct MatchPolyXNode {
    required_names: Vec<LabelOrAttr>,
    label: Label,
    new_label1: Option<Label>,
    new_label2: Option<Label>,
    x: u8,
    end: End,
    identity: f64,
}

impl MatchPolyXNode {
    const NAME: &'static str = "matching poly(X)";

    pub fn new(
        transform_expr: TransformExpr,
        x: u8,
        end: End,
        identity: f64,
    ) -> Self {
        transform_expr.check_size(1, 2, Self::NAME);
        transform_expr.check_same_str_type(Self::NAME);

        Self {
            required_names: vec![transform_expr.before(0).into()],
            label: transform_expr.before(0),
            new_label1: transform_expr.after_label(0, Self::NAME),
            new_label2: transform_expr.after_label(1, Self::NAME),
            x,
            end,
            identity,
        }
    }
}

impl GraphNode for MatchPolyXNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

        let string = read
            .substring(self.label.str_type, self.label.label)
            .map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: Self::NAME,
            })?;

        if let Some(cut_idx) = match_polyx(string, self.x, self.end, self.identity) {
            read.cut(
                self.label.str_type,
                self.label.label,
                self.new_label1.as_ref().map(|l| l.label),
                self.new_label2.as_ref().map(|l| l.label),
                cut_idx,
            )
            .map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: Self::NAME,
            })?;
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

const MATCH: i32 = 1i32;
const MISMATCH: i32 = -2i32;

fn match_polyx(s: &[u8], x: u8, end: End, identity_threshold: f64) -> Option<EndIdx> {
    let mut score = 0i32;
    let mut matches = 0;
    let mut max_score = 0i32;
    let mut max_idx = None;

    let f = |(i, c)| {
        score += if c == x { MATCH } else { MISMATCH };
        matches += (c == x) as usize;

        if score > max_score {
            max_score = score;
            max_idx = Some(i + 1);
        }
    };

    match end {
        Left => s.iter().cloned().enumerate().for_each(f),
        Right => s.iter().rev().cloned().enumerate().for_each(f),
    }

    if let Some(idx) = max_idx {
        let identity = (matches as f64) / (idx as f64);

        if identity >= identity_threshold {
            Some(EndIdx::from_end(end, idx))
        } else {
            None
        }
    } else {
        None
    }
}
