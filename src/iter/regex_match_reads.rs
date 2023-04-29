use regex::bytes::*;

use crate::inline_string::*;
use crate::iter::*;

pub struct RegexMatchReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    attr: Attr,
    regex: Regex,
}

impl<R: Reads> RegexMatchReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr, attr: Attr, regex: &str) -> Self {
        Self {
            reads,
            selector_expr,
            attr,
            regex: Regex::new(regex).unwrap(),
        }
    }
}

impl<R: Reads> Reads for RegexMatchReads<R> {
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();
        let cap_names = self
            .regex
            .capture_names()
            .filter_map(|name| name.map(|n| InlineString::new(n)))
            .collect::<Vec<_>>();
        let mut new_mappings = Vec::new();

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            let str_mappings = read.str_mappings_mut(self.attr.str_type).unwrap();
            let mapping = str_mappings.mapping(self.attr.label).unwrap();
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

            new_mappings
                .drain(..)
                .for_each(|(label, start, len)| str_mappings.add_mapping(Some(label), start, len));
            *str_mappings
                .mapping_mut(self.attr.label)
                .unwrap()
                .data_mut(self.attr.attr) = Data::Bool(matched);
        }

        reads
    }
}
