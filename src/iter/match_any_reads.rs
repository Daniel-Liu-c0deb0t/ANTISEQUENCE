use block_aligner::{cigar::*, scan_block::*, scores::*};

use crate::iter::*;

pub struct MatchAnyReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    label: Label,
    patterns: Patterns,
    match_type: MatchType,
}

impl<R: Reads> MatchAnyReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        label: Label,
        patterns: Patterns,
        match_type: MatchType,
    ) -> Self {
        Self {
            reads,
            selector_expr,
            label,
            patterns,
            match_type,
        }
    }
}

impl<R: Reads> Reads for MatchAnyReads<R> {
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();
        let mut aligner: Option<Box<dyn Aligner>> = None;

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            let str_mappings = read.str_mappings(self.label.str_type).unwrap();
            let mapping = str_mappings.mapping(self.label.label).unwrap();
            let substring = str_mappings.substring(mapping);

            if aligner.is_none() {
                match self.match_type {
                    MatchType::GlobalAln(_) => {
                        aligner = Some(Box::new(MatchAligner::<false, false>::new(
                            substring.len() * 2,
                        )));
                    }
                    MatchType::LocalAln(_) => {
                        aligner = Some(Box::new(MatchAligner::<true, false>::new(
                            substring.len() * 2,
                        )));
                    }
                    MatchType::PrefixAln(_) | MatchType::SuffixAln(_) => {
                        aligner = Some(Box::new(MatchAligner::<false, true>::new(
                            substring.len() * 2,
                        )));
                    }
                    _ => (),
                }
            }

            let mut max_matches = 0;
            let mut max_pattern = None;

            for pattern in self.patterns.patterns() {
                let pattern_str = pattern.expr.format(read, false);
                let pattern_len = pattern_str.len();

                use MatchType::*;
                let matches = match self.match_type {
                    Exact => {
                        if substring == pattern_str {
                            Some(pattern_str.len())
                        } else {
                            None
                        }
                    }
                    ExactPrefix => {
                        if pattern_len <= substring.len()
                            && &substring[..pattern_len] == &pattern_str
                        {
                            Some(pattern_str.len())
                        } else {
                            None
                        }
                    }
                    ExactSuffix => {
                        if pattern_len <= substring.len()
                            && &substring[substring.len() - pattern_len..] == &pattern_str
                        {
                            Some(pattern_str.len())
                        } else {
                            None
                        }
                    }
                    Hamming(t) => {
                        let t = t.get(pattern_len);
                        hamming(substring, &pattern_str, t)
                    }
                    HammingPrefix(t) => {
                        if pattern_len <= substring.len() {
                            let t = t.get(pattern_len);
                            hamming(&substring[..pattern_len], &pattern_str, t)
                        } else {
                            None
                        }
                    }
                    HammingSuffix(t) => {
                        if pattern_len <= substring.len() {
                            let t = t.get(pattern_len);
                            hamming(&substring[substring.len() - pattern_len..], &pattern_str, t)
                        } else {
                            None
                        }
                    }
                    GlobalAln(t) => {
                        let t = t.get(pattern_len);
                        aligner
                            .as_mut()
                            .unwrap()
                            .align(substring, &pattern_str, t, false)
                    }
                    LocalAln(t) => {
                        let t = t.get(pattern_len);
                        aligner
                            .as_mut()
                            .unwrap()
                            .align(substring, &pattern_str, t, false)
                    }
                    PrefixAln(t) => {
                        let t = t.get(pattern_len);
                        let len = substring.len().min(pattern_len * 2);
                        aligner
                            .as_mut()
                            .unwrap()
                            .align(&substring[..len], &pattern_str, t, true)
                    }
                    SuffixAln(t) => {
                        let t = t.get(pattern_len);
                        let len = substring.len().min(pattern_len * 2);
                        aligner.as_mut().unwrap().align(
                            &substring[substring.len() - len..],
                            &pattern_str,
                            t,
                            false,
                        )
                    }
                };

                if let Some(matches) = matches {
                    if matches > max_matches {
                        max_matches = matches;
                        max_pattern = Some((pattern_str, &pattern.attrs));

                        if max_matches >= pattern_len {
                            break;
                        }
                    }
                }
            }

            let str_mappings = read.str_mappings_mut(self.label.str_type).unwrap();
            let mapping = str_mappings.mapping_mut(self.label.label).unwrap();

            if let Some((pattern_str, pattern_attrs)) = max_pattern {
                *mapping.data_mut(self.patterns.pattern_name()) = Data::Bytes(pattern_str);

                for (&attr, data) in self.patterns.attr_names().iter().zip(pattern_attrs) {
                    *mapping.data_mut(attr) = data.clone();
                }
            } else {
                *mapping.data_mut(self.patterns.pattern_name()) = Data::Bool(false);
            }
        }

        reads
    }

    fn finish(&self) -> Result<()> {
        self.reads.finish()
    }
}

fn hamming(a: &[u8], b: &[u8], threshold: usize) -> Option<usize> {
    if a.len() != b.len() {
        return None;
    }

    let a_ptr = a.as_ptr();
    let b_ptr = b.as_ptr();
    let n = a.len();
    let mut res = 0;
    let mut i = 0;

    unsafe {
        while i < (n / 8) * 8 {
            let a_word = std::ptr::read_unaligned(a_ptr.add(i) as *const u64);
            let b_word = std::ptr::read_unaligned(b_ptr.add(i) as *const u64);

            let xor = a_word ^ b_word;
            let or1 = xor | (xor >> 1);
            let or2 = or1 | (or1 >> 2);
            let or3 = or2 | (or2 >> 4);
            let mask = or3 & 0x0101010101010101u64;
            res += mask.count_ones() as usize;

            i += 8;
        }

        while i < n {
            res += (*a_ptr.add(i) != *b_ptr.add(i)) as usize;
            i += 1;
        }
    }

    let matches = n - res;

    if matches >= threshold {
        Some(matches)
    } else {
        None
    }
}

trait Aligner {
    fn align(&mut self, a: &[u8], pattern: &[u8], threshold: usize, prefix: bool) -> Option<usize>;
}

struct MatchAligner<const LOCAL: bool, const PREFIX_SUFFIX: bool> {
    a_padded: PaddedBytes,
    b_padded: PaddedBytes,
    // always store trace
    block: Block<true, LOCAL, LOCAL, PREFIX_SUFFIX>,
    cigar: Cigar,
    len: usize,
}

impl<const LOCAL: bool, const PREFIX_SUFFIX: bool> MatchAligner<LOCAL, PREFIX_SUFFIX> {
    const MIN_SIZE: usize = 32;
    const MAX_SIZE: usize = 512;
    const GAP_OPEN: i8 = -2;
    const GAP_EXTEND: i8 = -1;

    pub fn new(len: usize) -> Self {
        let a_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
        let b_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
        let block = Block::<true, LOCAL, LOCAL, PREFIX_SUFFIX>::new(len, len, Self::MAX_SIZE);
        let cigar = Cigar::new(len, len);

        Self {
            a_padded,
            b_padded,
            block,
            cigar,
            len,
        }
    }

    fn resize_if_needed(&mut self, len: usize) {
        if len > self.len {
            self.a_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
            self.b_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
            self.block = Block::<true, LOCAL, LOCAL, PREFIX_SUFFIX>::new(len, len, Self::MAX_SIZE);
            self.cigar = Cigar::new(len, len);
            self.len = len;
        }
    }
}

impl<const LOCAL: bool, const PREFIX_SUFFIX: bool> Aligner for MatchAligner<LOCAL, PREFIX_SUFFIX> {
    fn align(&mut self, a: &[u8], pattern: &[u8], threshold: usize, prefix: bool) -> Option<usize> {
        self.resize_if_needed(a.len().max(pattern.len()));

        if prefix {
            self.a_padded.set_bytes_rev::<NucMatrix>(a, Self::MAX_SIZE);
            self.b_padded
                .set_bytes_rev::<NucMatrix>(pattern, Self::MAX_SIZE);
        } else {
            self.a_padded.set_bytes::<NucMatrix>(a, Self::MAX_SIZE);
            self.b_padded
                .set_bytes::<NucMatrix>(pattern, Self::MAX_SIZE);
        }

        let gaps = Gaps {
            open: Self::GAP_OPEN,
            extend: Self::GAP_EXTEND,
        };

        let min_size = if LOCAL || PREFIX_SUFFIX {
            Self::MAX_SIZE
        } else {
            Self::MIN_SIZE
        };

        self.block.align(
            &self.a_padded,
            &self.b_padded,
            &NW1,
            gaps,
            min_size..=Self::MAX_SIZE,
            pattern.len() as i32,
        );

        let res = self.block.res();
        self.block.trace().cigar_eq(
            &self.a_padded,
            &self.b_padded,
            res.query_idx,
            res.reference_idx,
            &mut self.cigar,
        );

        let mut matches = 0;

        for i in 0..self.cigar.len() {
            if let OpLen {
                op: Operation::Eq,
                len,
            } = self.cigar.get(i)
            {
                matches += len;
            }
        }

        if matches >= threshold {
            Some(matches)
        } else {
            None
        }
    }
}
