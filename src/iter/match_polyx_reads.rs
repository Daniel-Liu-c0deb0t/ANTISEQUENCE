use crate::iter::*;

pub struct MatchPolyXReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    label: Label,
    new_label1: Option<Label>,
    new_label2: Option<Label>,
    x: u8,
    end: End,
    identity: f64,
}

impl<R: Reads> MatchPolyXReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        x: u8,
        end: End,
        identity: f64,
    ) -> Self {
        transform_expr.check_size(1, 2, "matching poly(X)");
        transform_expr.check_same_str_type("matching poly(X)");

        Self {
            reads,
            selector_expr,
            label: transform_expr.before()[0].clone(),
            new_label1: transform_expr.after()[0].clone().map(|l| match l {
                LabelOrAttr::Label(l) => l,
                _ => panic!("Expected type.label after the \"->\" in the transform expression when matching poly(X)"),
            }),
            new_label2: transform_expr.after()[1].clone().map(|l| match l {
                LabelOrAttr::Label(l) => l,
                _ => panic!("Expected type.label after the \"->\" in the transform expression when matching poly(X)"),
            }),
            x,
            end,
            identity,
        }
    }
}

impl GraphNode for CountNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(read) = read else { panic!("Expected some read!") };

        for (c, n) in self.counts.iter().zip(&self.selector_exprs) {
            if n.eval_bool(&read).map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: Self::NAME,
            })? {
                c.fetch_add(1, Ordering::Relaxed);
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

impl<R: Reads> Reads for MatchPolyXReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "matching poly(X)",
                })?)
            {
                continue;
            }

            let string = read
                .substring(self.label.str_type, self.label.label)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "matching poly(X)",
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
                    context: "matching poly(X)",
                })?;
            }
        }

        Ok(reads)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()
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
