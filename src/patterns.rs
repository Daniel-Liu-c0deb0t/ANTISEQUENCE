use crate::inline_string::*;
use crate::read::*;

pub struct Patterns {
    pattern_name: InlineString,
    attr_names: Vec<InlineString>,
    patterns: Vec<Pattern>,
}

impl Patterns {
    pub fn new(name: InlineString, patterns: Vec<Vec<u8>>) -> Self {
        Self {
            pattern_name: name,
            attr_names: Vec::new(),
            patterns: patterns
                .into_iter()
                .map(|v| Pattern {
                    pattern: v,
                    attrs: Vec::new(),
                })
                .collect(),
        }
    }

    pub fn from_tsv(tsv: &[u8]) -> Self {
        let mut lines = tsv.split(|&b| b == b'\n');
        let mut names = lines
            .next()
            .unwrap()
            .split(|&b| b.is_ascii_whitespace())
            .filter_map(|s| {
                if s.len() > 0 {
                    Some(InlineString::new(s))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let mut patterns = Vec::new();

        for line in lines {
            let split = line
                .split(|&b| b.is_ascii_whitespace())
                .filter(|s| s.len() > 0)
                .collect::<Vec<_>>();

            if split.is_empty() {
                continue;
            }

            assert_eq!(split.len(), names.len());

            let pattern = split[0].to_owned();
            let attrs = split[1..]
                .iter()
                .map(|s| Data::from_bytes(s))
                .collect::<Vec<_>>();
            patterns.push(Pattern { pattern, attrs });
        }

        let pattern_name = names[0];
        names.remove(0);

        Self {
            pattern_name,
            attr_names: names,
            patterns,
        }
    }

    pub fn pattern_name(&self) -> InlineString {
        self.pattern_name
    }

    pub fn attr_names(&self) -> &[InlineString] {
        &self.attr_names
    }

    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }
}

pub struct Pattern {
    pub pattern: Vec<u8>,
    pub attrs: Vec<Data>,
}
