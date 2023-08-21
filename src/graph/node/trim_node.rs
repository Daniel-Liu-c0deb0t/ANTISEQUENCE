use crate::graph::*;

pub struct TrimNode {
    required_names: Vec<LabelOrAttr>,
    labels: Vec<Label>,
}

impl TrimNode {
    const NAME: &'static str = "TrimNode";

    /// Trim the intervals of the specified labels by splicing them out of the underlying strings.
    ///
    /// When an interval is trimmed, its length will be set to zero. All intersecting
    /// intervals will also be adjusted accordingly for the shortening.
    pub fn new(labels: Vec<Label>) -> Self {
        Self {
            required_names: labels.iter().cloned().map(|l| l.into()).collect(),
            labels,
        }
    }
}

impl GraphNode for TrimNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

        self.labels
            .iter()
            .try_for_each(|l| read.trim(l.str_type, l.label))
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
