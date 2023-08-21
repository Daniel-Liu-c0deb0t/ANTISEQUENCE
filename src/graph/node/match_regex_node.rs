use regex::bytes::*;

use thread_local::*;

use crate::inline_string::*;
use crate::graph::*;

pub struct MatchRegexNode {
    required_names: Vec<LabelOrAttr>,
    label: Label,
    attr: Option<Attr>,
    regex: Regex,
    regex_local: ThreadLocal<Regex>,
}

impl MatchRegexNode {
    const NAME: &'static str = "MatchRegexNode";

    /// Match a regex pattern in an interval.
    ///
    /// If named capture groups are used, then intervals are automatically created at the match
    /// locations, labeled by the names specified in the regex.
    ///
    /// The transform expression must have one input label and one output attribute.
    ///
    /// Example `transform_expr`: `tr!(seq1.* -> seq1.*.matched)`.
    /// This will match the regex pattern two `seq1.*` and set `seq1.*.matched` to a boolean
    /// indicating whether the regex matches.
    pub fn new(
        transform_expr: TransformExpr,
        regex: &str,
    ) -> Self {
        transform_expr.check_size(1, 1, Self::NAME);
        transform_expr.check_same_str_type(Self::NAME);

        Self {
            required_names: vec![transform_expr.before(0).into()],
            label: transform_expr.before(0),
            attr: transform_expr.after_attr(0, Self::NAME),
            regex: Regex::new(regex).unwrap_or_else(|e| panic!("Error compiling regex: {e}")),
            regex_local: ThreadLocal::new(),
        }
    }
}

impl GraphNode for MatchRegexNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

        let regex = self.regex_local.get_or(|| self.regex.clone());
        let cap_names = regex
            .capture_names()
            .filter_map(|name| name.map(|n| InlineString::new(n.as_bytes())))
            .collect::<Vec<_>>();
        let mut new_mappings = Vec::new();

        let string = read
            .substring(self.label.str_type, self.label.label)
            .map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: Self::NAME,
            })?;
        let matched;

        match regex.captures(string) {
            Some(caps) => {
                matched = true;

                new_mappings.extend(cap_names.iter().filter_map(|&name| {
                    caps.name(name.as_str()).map(|m| (name, m.start(), m.len()))
                }));
            }
            None => matched = false,
        }

        let str_mappings = read.str_mappings_mut(self.label.str_type).unwrap();
        let offset = str_mappings.mapping(self.label.label).unwrap().start;

        for (label, start, len) in new_mappings.drain(..) {
            str_mappings.add_mapping(Some(label), offset + start, len);
        }

        if let Some(attr) = &self.attr {
            // panic to make borrow checker happy
            *read
                .data_mut(attr.str_type, attr.label, attr.attr)
                .unwrap_or_else(|e| panic!("Error in {}: {e}", Self::NAME)) = Data::Bool(matched);
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
