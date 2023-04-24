use rustc_hash::FxHashMap;

use std::fmt;

use crate::inline_string::*;

pub use EndIdx::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EndIdx {
    LeftEnd(usize),
    RightEnd(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum StrType {
    Name,
    Seq,
    Name1,
    Seq1,
    Name2,
    Seq2,
    Index1,
    Index2,
}

#[derive(Debug, Clone)]
pub struct Read {
    str_mappings: Vec<(StrType, StrMappings)>,
}

#[derive(Debug, Clone)]
pub struct StrMappings {
    mappings: Vec<Mapping>,
    string: Vec<u8>,
    qual: Option<Vec<u8>>,
}

impl StrMappings {
    pub fn new(string: Vec<u8>) -> Self {
        Self {
            mappings: vec![Mapping::new_default(string.len())],
            string,
            qual: None,
        }
    }

    pub fn new_with_qual(string: Vec<u8>, qual: Vec<u8>) -> Self {
        Self {
            mappings: vec![Mapping::new_default(string.len())],
            string,
            qual: Some(qual),
        }
    }

    pub fn get_data(&self, label: InlineString, attr: InlineString) -> Option<&Data> {
        self.get_mapping(label).and_then(|m| m.get_data(attr))
    }

    pub fn get_mapping(&self, label: InlineString) -> Option<&Mapping> {
        self.mappings.iter().find(|&m| m.label == label)
    }

    pub fn add_mapping(&mut self, label: Option<InlineString>, start: usize, len: usize) {
        let Some(label) = label else {
            return;
        };
        if self.get_mapping(label).is_some() {
            panic!("Label already exists: {}", label);
        }
        self.mappings.push(Mapping::new(label, start, len));
    }

    pub fn string(&self) -> &[u8] {
        &self.string
    }

    pub fn qual(&self) -> Option<&[u8]> {
        self.qual.as_ref().map(|q| q.as_slice())
    }

    pub fn substring(&self, mapping: &Mapping) -> &[u8] {
        &self.string[mapping.start..mapping.start + mapping.len]
    }

    pub fn cut(
        &mut self,
        label: InlineString,
        new_label1: Option<InlineString>,
        new_label2: Option<InlineString>,
        cut_idx: EndIdx,
    ) {
        let (start, len) = {
            let mapping = self
                .get_mapping(label)
                .unwrap_or_else(|| panic!("Label not found in string: {}", label));
            (mapping.start, mapping.len)
        };

        match cut_idx {
            LeftEnd(idx) => {
                let cut = idx.min(len);
                self.add_mapping(new_label1, start, cut);
                self.add_mapping(new_label2, start + cut, len - cut);
            }
            RightEnd(idx) => {
                let cut = idx.min(len);
                self.add_mapping(new_label1, start, len - cut);
                self.add_mapping(new_label2, start + len - cut, cut);
            }
        }
    }

    pub fn trim(&mut self, label: InlineString) {
        let trimmed = self
            .get_mapping(label)
            .unwrap_or_else(|| panic!("Label not found in string: {}", label))
            .clone();

        self.mappings.iter_mut().for_each(|m| {
            use Intersection::*;
            match trimmed.intersect(m) {
                ABOverlap(len) => {
                    m.start = trimmed.start;
                    m.len -= len;
                }
                BAOverlap(len) => {
                    m.len -= len;
                }
                AInsideB => {
                    m.len -= trimmed.len;
                }
                BInsideA => {
                    m.start = trimmed.start;
                    m.len = 0;
                }
                Equal => {
                    m.len = 0;
                }
                ABeforeB => {
                    m.start -= trimmed.len;
                }
                BBeforeA => (),
            }
        });

        self.string
            .drain(trimmed.start..trimmed.start + trimmed.len);

        if let Some(qual) = &mut self.qual {
            qual.drain(trimmed.start..trimmed.start + trimmed.len);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    pub label: InlineString,
    pub start: usize,
    pub len: usize,
    pub data: FxHashMap<InlineString, Data>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Bool(bool),
    Int(usize),
    Bytes(Vec<u8>),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Intersection {
    ABOverlap(usize),
    BAOverlap(usize),
    AInsideB,
    BInsideA,
    Equal,
    ABeforeB,
    BBeforeA,
}

impl Mapping {
    pub fn new_default(len: usize) -> Self {
        Self {
            label: InlineString::new("*"),
            start: 0,
            len,
            data: FxHashMap::default(),
        }
    }

    pub fn new(label: InlineString, start: usize, len: usize) -> Self {
        Self {
            label,
            start,
            len,
            data: FxHashMap::default(),
        }
    }

    pub fn intersect(&self, b: &Self) -> Intersection {
        let a_start = self.start;
        let a_end = self.start + self.len;
        let b_start = b.start;
        let b_end = b.start + b.len;

        use Intersection::*;
        if a_start == b_start && a_end == b_end {
            Equal
        } else if a_start < b_start && b_end < a_end {
            BInsideA
        } else if b_start < a_start && a_end < b_end {
            AInsideB
        } else if a_start == b_start {
            if a_end > b_end {
                BAOverlap(b_end - a_start)
            } else {
                ABOverlap(a_end - b_start)
            }
        } else if a_end == b_end {
            if a_start > b_start {
                BAOverlap(b_end - a_start)
            } else {
                ABOverlap(a_end - b_start)
            }
        } else if a_start <= b_start && b_start < a_end {
            ABOverlap(a_end - b_start)
        } else if a_start < b_end && b_end <= a_end {
            BAOverlap(b_end - a_start)
        } else if a_end <= b_start {
            ABeforeB
        } else if b_end <= a_start {
            BBeforeA
        } else {
            unreachable!()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get_data(&self, attr: InlineString) -> Option<&Data> {
        self.data.get(&attr)
    }
}

impl Read {
    pub fn from_fastq(name: &[u8], seq: &[u8], qual: &[u8]) -> Self {
        let name = StrMappings::new(name.to_owned());
        let seq = StrMappings::new_with_qual(seq.to_owned(), qual.to_owned());

        Self {
            str_mappings: vec![(StrType::Name, name), (StrType::Seq, seq)],
        }
    }

    pub fn to_fastq(&self) -> (&[u8], &[u8], &[u8]) {
        let name = self.get_str_mappings(StrType::Name).unwrap();
        let seq = self.get_str_mappings(StrType::Seq).unwrap();
        (name.string(), seq.string(), seq.qual().unwrap())
    }

    pub fn get_str_mappings(&self, str_type: StrType) -> Option<&StrMappings> {
        self.str_mappings
            .iter()
            .find_map(|(t, m)| if *t == str_type { Some(m) } else { None })
    }

    pub fn get_str_mappings_mut(&mut self, str_type: StrType) -> Option<&mut StrMappings> {
        self.str_mappings
            .iter_mut()
            .find_map(|(t, m)| if *t == str_type { Some(m) } else { None })
    }

    pub fn cut(
        &mut self,
        str_type: StrType,
        label: InlineString,
        new_label1: Option<InlineString>,
        new_label2: Option<InlineString>,
        cut_idx: EndIdx,
    ) {
        self.get_str_mappings_mut(str_type)
            .unwrap()
            .cut(label, new_label1, new_label2, cut_idx);
    }

    pub fn trim(&mut self, str_type: StrType, label: InlineString) {
        self.get_str_mappings_mut(str_type).unwrap().trim(label);
    }
}

impl Data {
    pub fn as_bool(&self) -> bool {
        use Data::*;
        match self {
            Bool(x) => *x,
            Int(x) => *x > 0,
            Bytes(x) => !x.is_empty(),
            String(x) => !x.is_empty(),
        }
    }
}

impl fmt::Display for StrMappings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self
            .mappings
            .iter()
            .map(|m| m.label.len())
            .max()
            .unwrap()
            .max(4);

        for m in &self.mappings {
            let curr = if m.is_empty() {
                let str_len = self.string.len();
                format!("{: <str_len$}", "(empty)")
            } else {
                let mut c = vec![b' '; self.string.len()];
                c[m.start] = b'|';
                c[m.start + m.len - 1] = b'|';
                c[m.start + 1..m.start + m.len - 1].fill(b'-');
                String::from_utf8(c).unwrap()
            };
            write!(f, "{: <len$} {}", m.label.to_string(), curr)?;

            for (k, v) in &m.data {
                write!(f, " {}={}", k, v)?;
            }
            writeln!(f)?;
        }

        writeln!(
            f,
            "{: <len$} {}",
            "str",
            std::str::from_utf8(&self.string).unwrap()
        )?;

        if let Some(qual) = &self.qual {
            writeln!(
                f,
                "{: <len$} {}",
                "qual",
                std::str::from_utf8(&qual).unwrap()
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for Read {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (str_type, str_mapping) in &self.str_mappings {
            writeln!(f, "{}\n{}", str_type, str_mapping)?;
        }
        Ok(())
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Data::*;
        match self {
            Bool(x) => write!(f, "{}", x),
            Int(x) => write!(f, "{}", x),
            Bytes(x) => write!(f, "{}", std::str::from_utf8(x).unwrap()),
            String(x) => write!(f, "{}", x),
        }
    }
}

impl StrType {
    pub fn new(str_type: &str) -> Self {
        use StrType::*;
        match str_type {
            "name" => Name,
            "seq" => Seq,
            "name1" => Name1,
            "seq1" => Seq1,
            "name2" => Name2,
            "seq2" => Seq2,
            "index1" => Index1,
            "index2" => Index2,
            _ => panic!("Unknown string: {}", str_type),
        }
    }
}

impl fmt::Display for StrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use StrType::*;
        match self {
            Name => write!(f, "name"),
            Seq => write!(f, "seq"),
            Name1 => write!(f, "name1"),
            Seq1 => write!(f, "seq1"),
            Name2 => write!(f, "name2"),
            Seq2 => write!(f, "seq2"),
            Index1 => write!(f, "index1"),
            Index2 => write!(f, "index2"),
        }
    }
}
