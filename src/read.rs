pub struct Read {
    name: Mappings,
    seq: Mappings,
    qual: Mappings,
}

pub struct Mappings {
    mappings: Vec<Mapping>,
}

impl Mappings {
    pub fn new(mapping: Mapping) -> Self {
        Self { mappings: vec![mapping] }
    }
}

pub struct Mapping {
    label: String,
    string: Vec<u8>,
}

impl Mapping {
    pub fn new(string: Vec<u8>) -> Self {
        Self {
            label: "*".to_owned(),
            string,
        }
    }
}

impl Read {
    pub fn from_fastq(name: &[u8], seq: &[u8], qual: &[u8]) -> Self {
        let name = Mappings::new(Mapping::new(name.to_owned()));
        let seq = Mappings::new(Mapping::new(seq.to_owned()));
        let qual = Mappings::new(Mapping::new(qual.to_owned()));

        Self {
            name,
            seq,
            qual,
        }
    }
}

impl fmt::Display for Mapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.label, std::str::from_utf8(self.string).unwrap())
    }
}

impl fmt::Display for Mappings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut labels = String::new();
        let mut strings = String::new();

        for m in &self.mappings {
            let len = m.label.len().max(m.string.len()) + 1;
            labels.push_str(&format!("{: <len}", m.label));
            strings.push_str(&format!("{: <len}", m.string));
        }

        writeln!(f, "{}", labels)?;
        writeln!(f, "{}", strings)
    }
}

impl fmt::Display for Read {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "name\n{}", self.name)?;
        writeln!(f, "seq \n{}", self.seq)?;
        writeln!(f, "qual\n{}", self.qual)
    }
}
