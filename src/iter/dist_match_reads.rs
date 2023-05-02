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

        for read in reads.iter_mut().filter(|r| self.selector_expr.matches(r)) {
            let str_mappings = read.str_mappings_mut(self.label.str_type).unwrap();
            let mapping = str_mappings.mapping(self.label.label).unwrap();
            let substring = str_mappings.substring(mapping);

            let mut min_dist = std::usize::MAX;
            let mut min_pattern = None;

            for pattern in self.patterns.patterns() {
                use DistanceType::*;
                let matched = match self.dist_type {
                    Exact => {
                        if substring == pattern.pattern {
                            min_dist = 0;
                            min_pattern = Some(pattern);
                            break;
                        }
                    }
                    Hamming(t) => {
                        let t = t.get(pattern.pattern.len());
                        let dist = hamming(substring, &pattern.pattern, t);

                        if let Some(dist) = dist {
                            if dist < min_dist {
                                min_dist = dist;
                                min_pattern = Some(pattern);
                            }
                        }
                    }
                    GlobalAln(t) => {}
                };
            }

            let mapping = str_mappings.mapping_mut(self.label.label).unwrap();

            if let Some(pattern) = min_pattern {
                *mapping.data_mut(self.patterns.pattern_name()) =
                    Data::Bytes(pattern.pattern.to_owned());

                for (&attr, data) in self.patterns.attr_names().iter().zip(&pattern.attrs) {
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
