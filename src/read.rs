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

    pub fn get_data(&self, label: &str, attr: &str) -> Option<&Data> {
        self.get_mapping(label).and_then(|m| m.get_data(attr))
    }

    pub fn get_mapping(&self, label: &str) -> Option<&Mapping> {
        self.mappings.iter().find(|&m| m.label == label)
    }

    pub fn get_region(&self, mapping: &Mapping) -> &[u8] {
        &self.string[mapping.start..mapping.start + mapping.len]
    }

    pub fn trim(&mut self, label: &str) {
        let mapping = self.get_mapping(label).unwrap_or_else(|| panic!("Label not found in string: {}", label)).clone();

        self.mappings.iter_mut().for_each(|m| {
            use Intersection::*;
            match mapping.intersect(m) {
                BStart(len) => {
                    m.start += len;
                    m.len -= len;
                }
                BEnd(len) => {
                    m.len -= len;
                }
                AInsideB => {
                    m.len -= mapping.len;
                }
                BInsideA => {
                    m.start = mapping.start;
                    m.len = 0;
                }
                Equal => {
                    m.len = 0;
                }
                None => (),
            }
        });

        self.string.drain(mapping.start..mapping.start + mapping.len);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    pub label: String,
    pub start: usize,
    pub len: usize,
    pub data: FxHashMap<String, Data>,
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
    BStart(usize),
    BEnd(usize),
    AInsideB,
    BInsideA,
    Equal,
    None,
}

impl Mapping {
    pub fn new(label: &str, len: usize) -> Self {
        Self {
            label: label.to_owned(),
            start: 0,
            len,
            data: FxHashMap::default(),
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

    pub fn get_data(attr: &str) -> Option<&Data> {
        self.data.get(attr)
    }
}

impl Read {
    pub fn from_fastq(name: &[u8], seq: &[u8], qual: &[u8]) -> Self {
        let name = Mappings::new("name", name.to_owned());
        let seq = Mappings::new("*", seq.to_owned());
        let qual = Mappings::new("*", qual.to_owned());

        Self {
            name,
            seq,
            qual,
        }
    }

    pub fn to_fastq(&self) -> (&[u8], &[u8], &[u8]) {
        (&self.name.string, &self.seq.string, &self.qual.string)
    }

    pub fn trim(&mut self, label: &str) {
        self.name.trim(label);
        self.seq.trim(label);
        self.qual.trim(label);
    }
}

impl Data {
    pub fn as_bool(&self) -> bool {
        match self {
            Bool(x) => x,
            Int(x) => x > 0,
            Bytes(x) => !x.is_empty(),
            String(x) => !x.is_empty(),
        }
    }
}

impl fmt::Display for Mappings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.mappings.iter().map(|m| m.label.len()).max().unwrap();

        for m in &self.mappings {
            let curr = if m.is_empty() {
                let str_len = self.string.len();
                format!("{: <str_len}", "(empty)")
            } else {
                let mut c = " ".repeat(self.string.len());
                c[m.start] = '|';
                c[m.start + m.len - 1] = '|';
                c[m.start + 1..m.start + m.len - 1] = '-';
                c
            };
            write!(f, "{: <len} {}", m.label, curr)?;

            for (k, v) in &m.data {
                write!(f, " {}={}", k, v)?;
            }
            writeln!(f);
        }

        writeln!(f, "{: <len} {}", "", self.string)
    }
}

impl fmt::Display for Read {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "name\n{}", self.name)?;
        writeln!(f, "seq \n{}", self.seq)?;
        writeln!(f, "qual\n{}", self.qual)
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bool(x) => write!(f, "{}", x),
            Int(x) => write!(f, "{}", x),
            Bytes(x) => write!(f, "{}", std::str::from_utf8(x).unwrap()),
            String(x) => write!(f, "{}", x),
        }
    }
}
