use block_aligner::{cigar::*, scan_block::*, scores::*};

use crate::iter::*;

pub struct DistMatchReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    label: Label,
    patterns: Patterns,
    dist_type: DistanceType,
}

impl<R: Reads> DistMatchReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        label: Label,
        patterns: Patterns,
        dist_type: DistanceType,
    ) -> Self {
        Self {
            reads,
            selector_expr,
            label,
            patterns,
            dist_type,
        }
    }
}

impl<R: Reads> Reads for DistMatchReads<R> {
    fn next_chunk(&self) -> Vec<Read> {
        let mut reads = self.reads.next_chunk();
        let mut aligner = None;

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            let str_mappings = read.str_mappings(self.label.str_type).unwrap();
            let mapping = str_mappings.mapping(self.label.label).unwrap();
            let substring = str_mappings.substring(mapping);

            if aligner.is_none() {
                if let DistanceType::GlobalAln(_) = self.dist_type {
                    aligner = Some(MatchAligner::new(substring.len() * 2));
                }
            }

            let mut min_dist = std::usize::MAX;
            let mut min_pattern = None;

            for pattern in self.patterns.patterns() {
                let pattern_str = pattern.expr.format(read, false);

                use DistanceType::*;
                let dist = match self.dist_type {
                    Exact => {
                        if substring == pattern_str {
                            Some(0)
                        } else {
                            None
                        }
                    }
                    Hamming(t) => {
                        let t = t.get(pattern_str.len());
                        hamming(substring, &pattern_str, t)
                    }
                    GlobalAln(t) => {
                        let t = t.get(pattern_str.len());
                        aligner.as_mut().unwrap().align(substring, &pattern_str, t)
                    }
                };

                if let Some(dist) = dist {
                    if dist < min_dist {
                        min_dist = dist;
                        min_pattern = Some((pattern_str, &pattern.attrs));

                        if min_dist == 0 {
                            break;
                        }
                    }
                }
            }

            let str_mappings = read.str_mappings_mut(self.label.str_type).unwrap();
            let mapping = str_mappings.mapping_mut(self.label.label).unwrap();

            if let Some((pattern_str, pattern_attrs)) = min_pattern {
                *mapping.data_mut(self.patterns.pattern_name()) = Data::Bytes(pattern_str);

                for (&attr, data) in self.patterns.attr_names().iter().zip(pattern_attrs) {
                    *mapping.data_mut(attr) = data.clone();
                }
            } else {
                *mapping.data_mut(self.patterns.pattern_name()) = Data::Bytes(b"".to_vec());
            }
        }

        reads
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

    if res <= threshold {
        Some(res)
    } else {
        None
    }
}

struct MatchAligner {
    a_padded: PaddedBytes,
    b_padded: PaddedBytes,
    // store trace and global alignment
    block: Block<true, false>,
    cigar: Cigar,
    len: usize,
}

impl MatchAligner {
    const MIN_SIZE: usize = 32;
    const MAX_SIZE: usize = 256;
    const GAP_OPEN: i8 = -2;
    const GAP_EXTEND: i8 = -1;

    pub fn new(len: usize) -> Self {
        let a_padded = PaddedBytes::new::<ByteMatrix>(len, Self::MAX_SIZE);
        let b_padded = PaddedBytes::new::<ByteMatrix>(len, Self::MAX_SIZE);
        let block = Block::<true, false>::new(len, len, Self::MAX_SIZE);
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
            self.a_padded = PaddedBytes::new::<ByteMatrix>(len, Self::MAX_SIZE);
            self.b_padded = PaddedBytes::new::<ByteMatrix>(len, Self::MAX_SIZE);
            self.block = Block::<true, false>::new(len, len, Self::MAX_SIZE);
            self.cigar = Cigar::new(len, len);
            self.len = len;
        }
    }

    pub fn align(&mut self, a: &[u8], pattern: &[u8], threshold: usize) -> Option<usize> {
        self.resize_if_needed(a.len().max(pattern.len()));

        self.a_padded.set_bytes::<ByteMatrix>(a, Self::MAX_SIZE);
        self.b_padded
            .set_bytes::<ByteMatrix>(pattern, Self::MAX_SIZE);
        let gaps = Gaps {
            open: Self::GAP_OPEN,
            extend: Self::GAP_EXTEND,
        };
        self.block.align(
            &self.a_padded,
            &self.b_padded,
            &BYTES1,
            gaps,
            Self::MIN_SIZE..=Self::MAX_SIZE,
            0,
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

        let unmatched = pattern.len() - matches;

        if unmatched <= threshold {
            Some(unmatched)
        } else {
            None
        }
    }
}
