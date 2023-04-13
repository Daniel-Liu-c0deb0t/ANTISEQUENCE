pub struct Read {
    name: Mappings,
    seq: Mappings,
    qual: Mappings,
}

#[derive(Clone)]
pub struct Mappings {
    mappings: Vec<Mapping>,
    string: Vec<u8>,
}

impl Mappings {
    pub fn new(string: Vec<u8>) -> Self {
        Self { mappings: vec![Mapping::new(string.len())], string }
    }

    pub fn get(&self, label: &str) -> Option<&Mapping> {
        self.mappings.iter().find(|&m| m.label == label)
    }

    pub fn trim(&mut self, label: &str) {
        let mapping = self.get(label).unwrap_or_else(|| panic!("Label not found in string: {}", label)).clone();

        self.mappings.retain_mut(|m| {
            use Intersection::*;
            match mapping.intersect(m) {
                BStart(len) => {
                    m.start += len;
                    m.len -= len;
                    true
                }
                BEnd(len) => {
                    m.len -= len;
                    true
                }
                AInsideB => {
                    m.len -= mapping.len;
                    true
                }
                BInsideA => false,
                Equal => false,
                None => true,
            }
        });

        self.string.drain(mapping.start..mapping.start + mapping.len);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    label: String,
    start: usize,
    len: usize,
}

pub enum Intersection {
    BStart(usize),
    BEnd(usize),
    AInsideB,
    BInsideA,
    Equal,
    None,
}

impl Mapping {
    pub fn new(len: usize) -> Self {
        Self {
            label: "*".to_owned(),
            start: 0,
            len,
        }
    }

    pub fn intersect(&self, b: &Self) -> Intersection {
        let a_start = self.start;
        let a_end = self.start + self.len;
        let b_start = b.start;
        let b_end = b.start + b.len;

        if a_start == b_start && a_end == b_end {
            Intersection::Equal
        } else if a_start <= b_start && b_end <= a_end {
            Intersection::BInsideA
        } else if b_start <= a_start && a_end <= b_end {
            Intersection::AInsideB
        } else if a_start <= b_start && b_start < a_end {
            Intersection::BStart(a_end - b_start)
        } else if a_start < b_end && b_end <= a_end {
            Intersection::BEnd(b_end - a_start)
        } else {
            Intersection::None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Read {
    pub fn from_fastq(name: &[u8], seq: &[u8], qual: &[u8]) -> Self {
        let name = Mappings::new(name.to_owned());
        let seq = Mappings::new(seq.to_owned());
        let qual = Mappings::new(qual.to_owned());

        Self {
            name,
            seq,
            qual,
        }
    }

    pub fn to_fastq(&self) -> (&[u8], &[u8], &[u8]) {
        (&self.name.string, &self.seq.string, &self.qual.string)
    }

    pub fn trim_seq(&mut self, label: &str) {
        self.seq.trim(label);
        self.qual.trim(label);
    }

    pub fn trim_name(&mut self, label: &str) {
        self.name.trim(label);
    }
}

impl fmt::Display for Mappings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.mappings.iter().map(|m| m.label.len()).max().unwrap();

        for m in &self.mappings {
            let mut curr = " ".repeat(self.string.len() + 1);
            curr[m.start] = '|';
            curr[m.start + m.len] = '|';
            curr[m.start + 1..m.start + m.len] = '-';
            writeln!(f, "{: <len}{}", m.label, curr)?;
        }

        writeln!(f, "{: <len}{}", "str", self.string)
    }
}

impl fmt::Display for Read {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "name\n{}", self.name)?;
        writeln!(f, "seq \n{}", self.seq)?;
        writeln!(f, "qual\n{}", self.qual)
    }
}
