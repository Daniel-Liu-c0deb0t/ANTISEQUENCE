use regex::bytes::*;

use crate::inline_string::*;
use crate::iter::*;

pub struct MatchRegexReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    transform_expr: TransformExpr,
    regex: Regex,
}

impl<R: Reads> MatchRegexReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        regex: &str,
    ) -> Self {
        transform_expr.check_size(1, 1);
        transform_expr.check_same_str_type();

        Self {
            reads,
            selector_expr,
            transform_expr,
            regex: Regex::new(regex).unwrap(),
        }
    }
}

impl<R: Reads> Reads for MatchRegexReads<R> {
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();
        let cap_names = self
            .regex
            .capture_names()
            .filter_map(|name| name.map(|n| InlineString::new(n.as_bytes())))
            .collect::<Vec<_>>();
        let mut new_mappings = Vec::new();

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            if let Some(label_or_attr) = self.transform_expr.after()[0].as_ref() {
                let LabelOrAttr::Attr(after) = label_or_attr else {
                    panic!("Expected type.label.attr!")
                };

                let before = &self.transform_expr.before()[0];
                let str_mappings = read.str_mappings_mut(before.str_type).unwrap();
                let mapping = str_mappings.mapping(before.label).unwrap();
                let string = str_mappings.substring(&mapping);
                let matched;

                match self.regex.captures(string) {
                    Some(caps) => {
                        matched = true;

                        new_mappings.extend(cap_names.iter().filter_map(|&name| {
                            caps.name(name.as_str()).map(|m| (name, m.start(), m.len()))
                        }));
                    }
                    None => matched = false,
                }

                new_mappings.drain(..).for_each(|(label, start, len)| {
                    str_mappings.add_mapping(Some(label), start, len)
                });

                *read
                    .str_mappings_mut(after.str_type)
                    .unwrap()
                    .mapping_mut(after.label)
                    .unwrap()
                    .data_mut(after.attr) = Data::Bool(matched);
            }
        }

        reads
    }

    fn finish(&self) {
        self.reads.finish();
    }
}
