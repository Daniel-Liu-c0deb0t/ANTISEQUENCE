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

    pub fn data(&self, label: InlineString, attr: InlineString) -> Option<&Data> {
        self.mapping(label).and_then(|m| m.data(attr))
    }

    pub fn data_mut(&mut self, label: InlineString, attr: InlineString) -> Option<&mut Data> {
        self.mapping_mut(label).map(|m| m.data_mut(attr))
    }

    pub fn mapping(&self, label: InlineString) -> Option<&Mapping> {
        self.mappings.iter().find(|m| m.label == label)
    }

    pub fn mapping_mut(&mut self, label: InlineString) -> Option<&mut Mapping> {
        self.mappings.iter_mut().find(|m| m.label == label)
    }

    pub fn add_mapping(&mut self, label: Option<InlineString>, start: usize, len: usize) {
        let Some(label) = label else {
            return;
        };
        if self.mapping(label).is_some() {
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

    pub fn substring_qual(&self, mapping: &Mapping) -> Option<&[u8]> {
        self.qual
            .as_ref()
            .map(|q| &q[mapping.start..mapping.start + mapping.len])
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
                .mapping(label)
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

    pub fn set(&mut self, label: InlineString, new_str: &[u8], new_qual: Option<&[u8]>) {
        let prev = self
            .mapping(label)
            .unwrap_or_else(|| panic!("Label not found in string: {}", label))
            .clone();

        self.mappings.iter_mut().for_each(|m| {
            if m.label.bytes().all(|b| b == b'*') {
                if new_str.len() >= prev.len {
                    m.len += new_str.len() - prev.len;
                } else {
                    m.len -= prev.len - new_str.len();
                }

                return;
            }

            use Intersection::*;
            match prev.intersect(m) {
                ABOverlap(len) => {
                    if len > new_str.len() {
                        m.start = prev.start;
                        m.len -= len - new_str.len();
                    } else {
                        if new_str.len() >= prev.len {
                            m.start += new_str.len() - prev.len;
                        } else {
                            m.start -= prev.len - new_str.len();
                        }
                    }
                }
                BAOverlap(len) => {
                    if len > new_str.len() {
                        m.len -= len - new_str.len();
                    }
                }
                AInsideB => {
                    if new_str.len() >= prev.len {
                        m.len += new_str.len() - prev.len;
                    } else {
                        m.len -= prev.len - new_str.len();
                    }
                }
                BInsideA => {
                    m.start = m.start.min(prev.start + new_str.len());
                    m.len = m.len.min(prev.start + new_str.len() - m.start);
                }
                Equal => {
                    m.len = new_str.len();
                }
                ABeforeB => {
                    if new_str.len() >= prev.len {
                        m.start += new_str.len() - prev.len;
                    } else {
                        m.start -= prev.len - new_str.len();
                    }
                }
                BBeforeA => (),
            }
        });

        self.string
            .splice(prev.start..prev.start + prev.len, new_str.iter().cloned());

        if let Some(qual) = &mut self.qual {
            qual.splice(
                prev.start..prev.start + prev.len,
                new_qual.unwrap().iter().cloned(),
            );
        }
    }

    pub fn trim(&mut self, label: InlineString) {
        let trimmed = self
            .mapping(label)
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
    data: FxHashMap<InlineString, Data>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Bool(bool),
    UInt(usize),
    Bytes(Vec<u8>),
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
            label: InlineString::new(b"*"),
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

    pub fn data(&self, attr: InlineString) -> Option<&Data> {
        self.data.get(&attr)
    }

    pub fn data_mut(&mut self, attr: InlineString) -> &mut Data {
        self.data.entry(attr).or_insert_with(|| Data::Bool(false))
    }
}

impl Read {
    pub fn from_fastq1(name: &[u8], seq: &[u8], qual: &[u8]) -> Self {
        let name = StrMappings::new(name.to_owned());
        let seq = StrMappings::new_with_qual(seq.to_owned(), qual.to_owned());

        Self {
            str_mappings: vec![(StrType::Name1, name), (StrType::Seq1, seq)],
        }
    }

    pub fn from_fastq2(
        name1: &[u8],
        seq1: &[u8],
        qual1: &[u8],
        name2: &[u8],
        seq2: &[u8],
        qual2: &[u8],
    ) -> Self {
        let name1 = StrMappings::new(name1.to_owned());
        let seq1 = StrMappings::new_with_qual(seq1.to_owned(), qual1.to_owned());
        let name2 = StrMappings::new(name2.to_owned());
        let seq2 = StrMappings::new_with_qual(seq2.to_owned(), qual2.to_owned());

        Self {
            str_mappings: vec![
                (StrType::Name1, name1),
                (StrType::Seq1, seq1),
                (StrType::Name2, name2),
                (StrType::Seq2, seq2),
            ],
        }
    }

    pub fn to_fastq1(&self) -> (&[u8], &[u8], &[u8]) {
        let name = self.str_mappings(StrType::Name1).unwrap();
        let seq = self.str_mappings(StrType::Seq1).unwrap();
        (name.string(), seq.string(), seq.qual().unwrap())
    }

    pub fn to_fastq2(&self) -> ((&[u8], &[u8], &[u8]), (&[u8], &[u8], &[u8])) {
        let name1 = self.str_mappings(StrType::Name1).unwrap();
        let seq1 = self.str_mappings(StrType::Seq1).unwrap();
        let name2 = self.str_mappings(StrType::Name2).unwrap();
        let seq2 = self.str_mappings(StrType::Seq2).unwrap();
        (
            (name1.string(), seq1.string(), seq1.qual().unwrap()),
            (name2.string(), seq2.string(), seq2.qual().unwrap()),
        )
    }

    pub fn str_mappings(&self, str_type: StrType) -> Option<&StrMappings> {
        self.str_mappings
            .iter()
            .find_map(|(t, m)| if *t == str_type { Some(m) } else { None })
    }

    pub fn str_mappings_mut(&mut self, str_type: StrType) -> Option<&mut StrMappings> {
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
        self.str_mappings_mut(str_type)
            .unwrap()
            .cut(label, new_label1, new_label2, cut_idx);
    }

    pub fn set(
        &mut self,
        str_type: StrType,
        label: InlineString,
        new_str: &[u8],
        new_qual: Option<&[u8]>,
    ) {
        self.str_mappings_mut(str_type)
            .unwrap()
            .set(label, new_str, new_qual);
    }

    pub fn trim(&mut self, str_type: StrType, label: InlineString) {
        self.str_mappings_mut(str_type).unwrap().trim(label);
    }
}

impl Data {
    pub fn as_bool(&self) -> bool {
        use Data::*;
        match self {
            Bool(x) => *x,
            UInt(x) => *x > 0,
            Bytes(x) => !x.is_empty(),
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
            let curr = if m.len == 0 {
                let mut c = vec![b' '; self.string.len() + 1];
                c[m.start] = b'|';
                String::from_utf8(c).unwrap()
            } else {
                let mut c = vec![b' '; self.string.len() + 1];
                c[m.start] = b'[';
                c[m.start + m.len - 1] = b']';
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
            writeln!(f, "{}:\n{}", str_type, str_mapping)?;
        }
        Ok(())
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Data::*;
        match self {
            Bool(x) => write!(f, "{}", x),
            UInt(x) => write!(f, "{}", x),
            Bytes(x) => write!(f, "{}", std::str::from_utf8(x).unwrap()),
        }
    }
}

impl StrType {
    pub fn new(str_type: &[u8]) -> Self {
        use StrType::*;
        match str_type {
            b"name1" => Name1,
            b"seq1" => Seq1,
            b"name2" => Name2,
            b"seq2" => Seq2,
            b"index1" => Index1,
            b"index2" => Index2,
            _ => panic!("Unknown string: {}", std::str::from_utf8(str_type).unwrap()),
        }
    }
}

impl fmt::Display for StrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use StrType::*;
        match self {
            Name1 => write!(f, "name1"),
            Seq1 => write!(f, "seq1"),
            Name2 => write!(f, "name2"),
            Seq2 => write!(f, "seq2"),
            Index1 => write!(f, "index1"),
            Index2 => write!(f, "index2"),
        }
    }
}
