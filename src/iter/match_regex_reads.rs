use regex::bytes::*;

use thread_local::*;

use crate::inline_string::*;
use crate::iter::*;

pub struct MatchRegexReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    label: Label,
    attr: Option<Attr>,
    regex: Regex,
    regex_local: ThreadLocal<Regex>,
}

impl<R: Reads> MatchRegexReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        regex: &str,
    ) -> Self {
        transform_expr.check_size(1, 1, "matching regex in reads");
        transform_expr.check_same_str_type("matching regex in reads");

        Self {
            reads,
            selector_expr,
            label: transform_expr.before()[0].clone(),
            attr: transform_expr.after()[0].clone().map(|a| match a {
                LabelOrAttr::Attr(a) => a,
                _ => panic!("Expected type.label.attr after the \"->\" in the transform expression when matching regex"),
            }),
            regex: Regex::new(regex).unwrap_or_else(|e| panic!("Error compiling regex: {e}")),
            regex_local: ThreadLocal::new(),
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

impl<R: Reads> Reads for MatchRegexReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;
        let regex = self.regex_local.get_or(|| self.regex.clone());
        let cap_names = regex
            .capture_names()
            .filter_map(|name| name.map(|n| InlineString::new(n.as_bytes())))
            .collect::<Vec<_>>();
        let mut new_mappings = Vec::new();

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "matching regex",
                })?)
            {
                continue;
            }

            let string = read
                .substring(self.label.str_type, self.label.label)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "matching regex",
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
                // panic to make borrow checker happy
                str_mappings.add_mapping(Some(label), offset + start, len);
            }

            if let Some(attr) = &self.attr {
                // panic to make borrow checker happy
                *read
                    .data_mut(attr.str_type, attr.label, attr.attr)
                    .unwrap_or_else(|e| panic!("Error matching regex: {e}")) = Data::Bool(matched);
            }
        }

        Ok(reads)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()
    }
}
