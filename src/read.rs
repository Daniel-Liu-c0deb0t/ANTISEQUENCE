use rustc_hash::FxHashMap;

use std::fmt;
use std::sync::Arc;

use crate::errors::{self, Name, NameError};
use crate::fastq::Origin;
use crate::inline_string::*;
use crate::normalize_reads::*;

pub use End::*;
pub use EndIdx::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EndIdx {
    LeftEnd(usize),
    RightEnd(usize),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum End {
    Left,
    Right,
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

    // tracks where this string came from
    origin: Arc<Origin>,
    idx: usize,
}

impl StrMappings {
    pub fn new(string: Vec<u8>, origin: Arc<Origin>, idx: usize) -> Self {
        Self {
            mappings: vec![Mapping::new_default(string.len())],
            string,
            qual: None,
            origin,
            idx,
        }
    }

    pub fn new_with_qual(string: Vec<u8>, qual: Vec<u8>, origin: Arc<Origin>, idx: usize) -> Self {
        Self {
            mappings: vec![Mapping::new_default(string.len())],
            string,
            qual: Some(qual),
            origin,
            idx,
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

    pub fn add_mapping(
        &mut self,
        label: Option<InlineString>,
        start: usize,
        len: usize,
    ) -> Result<(), NameError> {
        let Some(label) = label else {
            return Ok(());
        };
        if self.mapping(label).is_some() {
            Err(NameError::Duplicate(Name::Label(label)))?
        }
        self.mappings.push(Mapping::new(label, start, len));
        Ok(())
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
    ) -> Result<(), NameError> {
        let (start, len) = {
            let mapping = self
                .mapping(label)
                .ok_or_else(|| NameError::NotInRead(Name::Label(label)))?;
            (mapping.start, mapping.len)
        };

        match cut_idx {
            LeftEnd(idx) => {
                let cut = idx.min(len);
                self.add_mapping(new_label1, start, cut)?;
                self.add_mapping(new_label2, start + cut, len - cut)?;
            }
            RightEnd(idx) => {
                let cut = idx.min(len);
                self.add_mapping(new_label1, start, len - cut)?;
                self.add_mapping(new_label2, start + len - cut, cut)?;
            }
        }

        Ok(())
    }

    pub fn intersect(
        &mut self,
        label1: InlineString,
        label2: InlineString,
        new_label: Option<InlineString>,
    ) -> Result<(), NameError> {
        let mapping1 = self
            .mapping(label1)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label1)))?;
        let mapping2 = self
            .mapping(label2)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label2)))?;

        if let Some((start, len)) = mapping1.intersection_interval(mapping2) {
            self.add_mapping(new_label, start, len)?;
        }

        Ok(())
    }

    pub fn union(
        &mut self,
        label1: InlineString,
        label2: InlineString,
        new_label: Option<InlineString>,
    ) -> Result<(), NameError> {
        let mapping1 = self
            .mapping(label1)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label1)))?;
        let mapping2 = self
            .mapping(label2)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label2)))?;

        let (start, len) = mapping1.union_interval(mapping2);
        self.add_mapping(new_label, start, len)?;

        Ok(())
    }

    pub fn set(
        &mut self,
        label: InlineString,
        new_str: &[u8],
        new_qual: Option<&[u8]>,
    ) -> Result<(), NameError> {
        let prev = self
            .mapping(label)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label)))?
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

        Ok(())
    }

    pub fn norm(&mut self, label: InlineString, short_len: usize, long_len: usize) -> Result<(), NameError>
    {
        let normalized = self
            .mapping(label)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label)))?
            .clone();

        let mut length_diff = long_len - normalized.len;

        let extra_len = log4_roundup(long_len - short_len + 1);

        let normed_len = long_len - normalized.len + extra_len;

        self.mappings.iter_mut().for_each(|m| {
            use Intersection::*;
            match normalized.intersect(m) {
                BAOverlap(_) | ABOverlap(_) | AInsideB | ABeforeB | Equal => m.len += normed_len,
                _ => (),
            }
        });

        for _ in 0..length_diff {
            self.string.insert(normalized.start + normalized.len, b'A');
        }

        for _ in 0..extra_len {
            let nuc = NUC_MAP.get(length_diff & (usize::MAX & 3)).unwrap();
            length_diff >>= 2;

            self.string.insert(self.string.len(), *nuc)
        }

        if let Some(qual) = &mut self.qual {
            for _ in 0..normed_len {
                qual.insert(normalized.start + normalized.len, b'#');
            }
        }

        Ok(())
    }

    pub fn trim(&mut self, label: InlineString) -> Result<(), NameError> {
        let trimmed = self
            .mapping(label)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label)))?
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

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    pub label: InlineString,
    pub start: usize,
    pub len: usize,
    data: FxHashMap<InlineString, Data>,
}

#[derive(Clone, PartialEq)]
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

    pub fn intersection_interval(&self, b: &Self) -> Option<(usize, usize)> {
        let a_start = self.start;
        let a_end = self.start + self.len;
        let b_start = b.start;
        let b_end = b.start + b.len;

        if (b_start <= a_start && a_start < b_end) || (a_start <= b_start && b_start < a_end) {
            let start = a_start.max(b_start);
            let len = a_end.min(b_end) - start;
            Some((start, len))
        } else {
            None
        }
    }

    pub fn union_interval(&self, b: &Self) -> (usize, usize) {
        let a_start = self.start;
        let a_end = self.start + self.len;
        let b_start = b.start;
        let b_end = b.start + b.len;

        let start = a_start.min(b_start);
        let len = a_end.max(b_end) - start;
        (start, len)
    }

    pub fn data(&self, attr: InlineString) -> Option<&Data> {
        self.data.get(&attr)
    }

    pub fn data_mut(&mut self, attr: InlineString) -> &mut Data {
        self.data.entry(attr).or_insert_with(|| Data::Bool(false))
    }
}

impl Read {
    pub fn from_fastq1(
        name: &[u8],
        seq: &[u8],
        qual: &[u8],
        origin: Arc<Origin>,
        idx: usize,
    ) -> Self {
        let name = StrMappings::new(name.to_owned(), Arc::clone(&origin), idx);
        let seq = StrMappings::new_with_qual(seq.to_owned(), qual.to_owned(), origin, idx);

        Self {
            str_mappings: vec![(StrType::Name1, name), (StrType::Seq1, seq)],
        }
    }

    pub fn from_fastq2(
        name1: &[u8],
        seq1: &[u8],
        qual1: &[u8],
        origin1: Arc<Origin>,
        idx1: usize,
        name2: &[u8],
        seq2: &[u8],
        qual2: &[u8],
        origin2: Arc<Origin>,
        idx2: usize,
    ) -> Self {
        let name1 = StrMappings::new(name1.to_owned(), Arc::clone(&origin1), idx1);
        let seq1 = StrMappings::new_with_qual(seq1.to_owned(), qual1.to_owned(), origin1, idx1);
        let name2 = StrMappings::new(name2.to_owned(), Arc::clone(&origin2), idx2);
        let seq2 = StrMappings::new_with_qual(seq2.to_owned(), qual2.to_owned(), origin2, idx2);

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

    pub fn to_fastq2(&self) -> Result<((&[u8], &[u8], &[u8]), (&[u8], &[u8], &[u8])), NameError> {
        let name1 = self.str_mappings(StrType::Name1).unwrap();
        let seq1 = self.str_mappings(StrType::Seq1).unwrap();
        let name2 = self
            .str_mappings(StrType::Name2)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(StrType::Name2)))?;
        let seq2 = self.str_mappings(StrType::Seq2).unwrap();
        Ok((
            (name1.string(), seq1.string(), seq1.qual().unwrap()),
            (name2.string(), seq2.string(), seq2.qual().unwrap()),
        ))
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

    pub fn mapping(&self, str_type: StrType, label: InlineString) -> Result<&Mapping, NameError> {
        self.str_mappings(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .mapping(label)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label)))
    }

    pub fn mapping_mut(
        &mut self,
        str_type: StrType,
        label: InlineString,
    ) -> Result<&mut Mapping, NameError> {
        self.str_mappings_mut(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .mapping_mut(label)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label)))
    }

    pub fn data(
        &self,
        str_type: StrType,
        label: InlineString,
        attr: InlineString,
    ) -> Result<&Data, NameError> {
        self.str_mappings(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .mapping(label)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label)))?
            .data(attr)
            .ok_or_else(|| NameError::NotInRead(Name::Attr(attr)))
    }

    pub fn data_mut(
        &mut self,
        str_type: StrType,
        label: InlineString,
        attr: InlineString,
    ) -> Result<&mut Data, NameError> {
        Ok(self
            .str_mappings_mut(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .mapping_mut(label)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label)))?
            .data_mut(attr))
    }

    pub fn substring(&self, str_type: StrType, label: InlineString) -> Result<&[u8], NameError> {
        let str_mappings = self
            .str_mappings(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?;
        let mapping = str_mappings
            .mapping(label)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label)))?;
        Ok(str_mappings.substring(mapping))
    }

    pub fn substring_qual(
        &self,
        str_type: StrType,
        label: InlineString,
    ) -> Result<Option<&[u8]>, NameError> {
        let str_mappings = self
            .str_mappings(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?;
        let mapping = str_mappings
            .mapping(label)
            .ok_or_else(|| NameError::NotInRead(Name::Label(label)))?;
        Ok(str_mappings.substring_qual(mapping))
    }

    pub fn cut(
        &mut self,
        str_type: StrType,
        label: InlineString,
        new_label1: Option<InlineString>,
        new_label2: Option<InlineString>,
        cut_idx: EndIdx,
    ) -> Result<(), NameError> {
        self.str_mappings_mut(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .cut(label, new_label1, new_label2, cut_idx)
    }

    pub fn intersect(
        &mut self,
        str_type: StrType,
        label1: InlineString,
        label2: InlineString,
        new_label: Option<InlineString>,
    ) -> Result<(), NameError> {
        self.str_mappings_mut(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .intersect(label1, label2, new_label)
    }

    pub fn union(
        &mut self,
        str_type: StrType,
        label1: InlineString,
        label2: InlineString,
        new_label: Option<InlineString>,
    ) -> Result<(), NameError> {
        self.str_mappings_mut(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .union(label1, label2, new_label)
    }

    pub fn set(
        &mut self,
        str_type: StrType,
        label: InlineString,
        new_str: &[u8],
        new_qual: Option<&[u8]>,
    ) -> Result<(), NameError> {
        self.str_mappings_mut(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .set(label, new_str, new_qual)
    }

    pub fn norm(
        &mut self,
        str_type: StrType,
        label: InlineString,
        short_len: usize,
        long_len: usize,
    ) -> Result<(), NameError>
    {
        self.str_mappings_mut(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .norm(label, short_len, long_len)
    }

    pub fn trim(&mut self, str_type: StrType, label: InlineString) -> Result<(), NameError> {
        self.str_mappings_mut(str_type)
            .ok_or_else(|| NameError::NotInRead(Name::StrType(str_type)))?
            .trim(label)
    }

    pub fn first_idx(&self) -> usize {
        self.str_mappings.iter().map(|(_, s)| s.idx).min().unwrap()
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

    pub fn as_uint(&self) -> Result<usize, NameError> {
        use Data::*;
        match self {
            Bool(x) => Ok(if *x { 1 } else { 0 }),
            UInt(x) => Ok(*x),
            Bytes(_) => Err(NameError::Type("bool or uint", self.clone())),
        }
    }

    pub fn len(&self) -> Result<usize, NameError> {
        use Data::*;
        match self {
            Bool(_) => Err(NameError::Type("bytes", self.clone())),
            UInt(_) => Err(NameError::Type("bytes", self.clone())),
            Bytes(x) => Ok(x.len()),
        }
    }
}

impl EndIdx {
    pub fn from_end(end: End, idx: usize) -> Self {
        match end {
            Left => LeftEnd(idx),
            Right => RightEnd(idx),
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
                c[m.start] = b'.';
                String::from_utf8(c).unwrap()
            } else {
                let mut c = vec![b' '; self.string.len() + 1];
                c[m.start..m.start + m.len].fill(b'-');
                c[m.start] = b'|';
                c[m.start + m.len - 1] = b'|';
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

        writeln!(f, "(from record {} in {})", self.idx, &*self.origin)?;

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

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Data::*;
        match self {
            Bool(x) => write!(f, "bool {}", x),
            UInt(x) => write!(f, "uint {}", x),
            Bytes(x) => write!(f, "bytes \"{}\"", std::str::from_utf8(x).unwrap()),
        }
    }
}

impl StrType {
    pub fn new(str_type: &[u8]) -> Result<Self, errors::Error> {
        use StrType::*;
        match str_type {
            b"name1" => Ok(Name1),
            b"seq1" => Ok(Seq1),
            b"name2" => Ok(Name2),
            b"seq2" => Ok(Seq2),
            b"index1" => Ok(Index1),
            b"index2" => Ok(Index2),
            _ => Err(errors::Error::Parse {
                string: errors::utf8(str_type),
                context: errors::utf8(str_type),
                reason: "not a known valid string type. Expected \"name1\", \"seq1\", etc.",
            }),
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
