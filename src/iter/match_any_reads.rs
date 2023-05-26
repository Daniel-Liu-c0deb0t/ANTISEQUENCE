use block_aligner::{cigar::*, scan_block::*, scores::*};

use crate::iter::*;

pub struct MatchAnyReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    label: Label,
    new_labels: [Option<Label>; 3],
    patterns: Patterns,
    match_type: MatchType,
}

impl<R: Reads> MatchAnyReads<R> {
    pub fn new(
        reads: R,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        patterns: Patterns,
        match_type: MatchType,
    ) -> Self {
        let mut new_labels = [None, None, None];

        transform_expr.check_size(1, match_type.num_mappings(), "matching patterns");
        for i in 0..match_type.num_mappings() {
            new_labels[i] = transform_expr.after()[i].clone().map(|l| match l {
                LabelOrAttr::Label(l) => l,
                _ => panic!("Expected type.label after the \"->\" in the transform expression when matching patterns"),
            });
        }
        transform_expr.check_same_str_type("matching patterns");

        Self {
            reads,
            selector_expr,
            label: transform_expr.before()[0].clone(),
            new_labels,
            patterns,
            match_type,
        }
    }
}

impl<R: Reads> Reads for MatchAnyReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;
        let mut aligner: Option<Box<dyn Aligner>> = None;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "matching patterns",
                })?)
            {
                continue;
            }

            let string = read
                .substring(self.label.str_type, self.label.label)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "matching patterns",
                })?;

            if aligner.is_none() {
                match self.match_type {
                    MatchType::GlobalAln(_) => {
                        aligner = Some(Box::new(MatchAligner::<false, false, false>::new(
                            string.len() * 2,
                        )));
                    }
                    MatchType::LocalAln { .. } => {
                        aligner = Some(Box::new(MatchAligner::<true, false, true>::new(
                            string.len() * 2,
                        )));
                    }
                    MatchType::PrefixAln { .. } | MatchType::SuffixAln { .. } => {
                        aligner = Some(Box::new(MatchAligner::<false, true, true>::new(
                            string.len() * 2,
                        )));
                    }
                    _ => (),
                }
            }

            let mut max_matches = 0;
            let mut max_pattern = None;
            let mut max_cut_pos1 = 0;
            let mut max_cut_pos2 = 0;

            for pattern in self.patterns.patterns() {
                let pattern_str =
                    pattern
                        .expr
                        .format(read, false)
                        .map_err(|e| Error::NameError {
                            source: e,
                            read: read.clone(),
                            context: "matching patterns",
                        })?;
                let pattern_len = pattern_str.len();

                use MatchType::*;
                let matches = match self.match_type {
                    Exact => {
                        if string == pattern_str {
                            Some((pattern_len, pattern_len, 0))
                        } else {
                            None
                        }
                    }
                    ExactPrefix => {
                        if pattern_len <= string.len() && &string[..pattern_len] == &pattern_str {
                            Some((pattern_len, pattern_len, 0))
                        } else {
                            None
                        }
                    }
                    ExactSuffix => {
                        if pattern_len <= string.len()
                            && &string[string.len() - pattern_len..] == &pattern_str
                        {
                            Some((pattern_len, string.len() - pattern_len, 0))
                        } else {
                            None
                        }
                    }
                    Hamming(t) => {
                        let t = t.get(pattern_len);
                        hamming(string, &pattern_str, t).map(|m| (m, pattern_len, 0))
                    }
                    HammingPrefix(t) => {
                        if pattern_len <= string.len() {
                            let t = t.get(pattern_len);
                            hamming(&string[..pattern_len], &pattern_str, t)
                                .map(|m| (m, pattern_len, 0))
                        } else {
                            None
                        }
                    }
                    HammingSuffix(t) => {
                        if pattern_len <= string.len() {
                            let t = t.get(pattern_len);
                            hamming(&string[string.len() - pattern_len..], &pattern_str, t)
                                .map(|m| (m, string.len() - pattern_len, 0))
                        } else {
                            None
                        }
                    }
                    GlobalAln(identity) => aligner
                        .as_mut()
                        .unwrap()
                        .align(string, &pattern_str, identity, identity, false)
                        .map(|(m, _, end_idx)| (m, end_idx, 0)),
                    LocalAln { identity, overlap } => aligner.as_mut().unwrap().align(
                        string,
                        &pattern_str,
                        identity,
                        overlap,
                        false,
                    ),
                    PrefixAln { identity, overlap } => {
                        let additional =
                            ((1.0 - identity).max(0.0) * (pattern_len as f64)).ceil() as usize;
                        let len = string.len().min(pattern_len + additional);
                        aligner
                            .as_mut()
                            .unwrap()
                            .align(&string[..len], &pattern_str, identity, overlap, true)
                            .map(|(m, _, end_idx)| (m, end_idx, 0))
                    }
                    SuffixAln { identity, overlap } => {
                        let additional =
                            ((1.0 - identity).max(0.0) * (pattern_len as f64)).ceil() as usize;
                        let len = string.len().min(pattern_len + additional);
                        aligner
                            .as_mut()
                            .unwrap()
                            .align(
                                &string[string.len() - len..],
                                &pattern_str,
                                identity,
                                overlap,
                                false,
                            )
                            .map(|(m, start_idx, _)| (m, string.len() - len + start_idx, 0))
                    }
                };

                if let Some((matches, cut_pos1, cut_pos2)) = matches {
                    if matches > max_matches {
                        max_matches = matches;
                        max_pattern = Some((pattern_str, &pattern.attrs));
                        max_cut_pos1 = cut_pos1;
                        max_cut_pos2 = cut_pos2;

                        if max_matches >= pattern_len {
                            break;
                        }
                    }
                }
            }

            let mapping = read
                .mapping_mut(self.label.str_type, self.label.label)
                .unwrap();

            if let Some((pattern_str, pattern_attrs)) = max_pattern {
                *mapping.data_mut(self.patterns.pattern_name()) = Data::Bytes(pattern_str);

                for (&attr, data) in self.patterns.attr_names().iter().zip(pattern_attrs) {
                    *mapping.data_mut(attr) = data.clone();
                }

                match self.match_type.num_mappings() {
                    1 => {
                        let start = mapping.start;
                        let str_mappings = read.str_mappings_mut(self.label.str_type).unwrap();
                        // panic to make borrow checker happy
                        str_mappings
                            .add_mapping(
                                self.new_labels[0].as_ref().map(|l| l.label),
                                start,
                                max_cut_pos1,
                            )
                            .unwrap_or_else(|e| panic!("Error matching patterns: {e}"));
                    }
                    2 => {
                        read.cut(
                            self.label.str_type,
                            self.label.label,
                            self.new_labels[0].as_ref().map(|l| l.label),
                            self.new_labels[1].as_ref().map(|l| l.label),
                            LeftEnd(max_cut_pos1),
                        )
                        .unwrap_or_else(|e| panic!("Error matching patterns: {e}"));
                    }
                    3 => {
                        let offset = mapping.start;
                        let mapping_len = mapping.len;

                        let str_mappings = read.str_mappings_mut(self.label.str_type).unwrap();
                        // panic to make borrow checker happy
                        str_mappings
                            .add_mapping(
                                self.new_labels[0].as_ref().map(|l| l.label),
                                offset,
                                max_cut_pos1,
                            )
                            .unwrap_or_else(|e| panic!("Error matching patterns: {e}"));
                        str_mappings
                            .add_mapping(
                                self.new_labels[1].as_ref().map(|l| l.label),
                                offset + max_cut_pos1,
                                max_cut_pos2 - max_cut_pos1,
                            )
                            .unwrap_or_else(|e| panic!("Error matching patterns: {e}"));
                        str_mappings
                            .add_mapping(
                                self.new_labels[2].as_ref().map(|l| l.label),
                                offset + max_cut_pos2,
                                mapping_len - max_cut_pos2,
                            )
                            .unwrap_or_else(|e| panic!("Error matching patterns: {e}"));
                    }
                    _ => unreachable!(),
                }
            } else {
                *mapping.data_mut(self.patterns.pattern_name()) = Data::Bool(false);
            }
        }

        Ok(reads)
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
    fn align(
        &mut self,
        read: &[u8],
        pattern: &[u8],
        identity_threshold: f64,
        overlap_threshold: f64,
        prefix: bool,
    ) -> Option<(usize, usize, usize)>;
}

struct MatchAligner<const LOCAL: bool, const PREFIX_SUFFIX: bool, const LOCAL_PREFIX_SUFFIX: bool> {
    read_vec: Vec<u8>,
    read_padded: PaddedBytes,
    pattern_padded: PaddedBytes,
    matrix: NucMatrix,
    // always store trace
    block: Block<true, LOCAL_PREFIX_SUFFIX, LOCAL, PREFIX_SUFFIX>,
    cigar: Cigar,
    len: usize,
}

impl<const LOCAL: bool, const PREFIX_SUFFIX: bool, const LOCAL_PREFIX_SUFFIX: bool>
    MatchAligner<LOCAL, PREFIX_SUFFIX, LOCAL_PREFIX_SUFFIX>
{
    const MIN_SIZE: usize = 32;
    const MAX_SIZE: usize = 512;
    const GAP_OPEN: i8 = -2;
    const GAP_EXTEND: i8 = -1;

    pub fn new(len: usize) -> Self {
        assert_eq!(LOCAL_PREFIX_SUFFIX, LOCAL || PREFIX_SUFFIX);

        let read_vec = Vec::with_capacity(len);
        let read_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
        let pattern_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
        let mut matrix = NucMatrix::new_simple(1, -1);

        // use 'X' as padding
        for c in [b'A', b'C', b'G', b'T', b'N'] {
            matrix.set(c, b'X', 0);
        }

        let block =
            Block::<true, LOCAL_PREFIX_SUFFIX, LOCAL, PREFIX_SUFFIX>::new(len, len, Self::MAX_SIZE);
        let cigar = Cigar::new(len, len);

        Self {
            read_vec,
            read_padded,
            pattern_padded,
            matrix,
            block,
            cigar,
            len,
        }
    }

    fn resize_if_needed(&mut self, len: usize) {
        if len > self.len {
            self.read_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
            self.pattern_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
            self.block = Block::<true, LOCAL_PREFIX_SUFFIX, LOCAL, PREFIX_SUFFIX>::new(
                len,
                len,
                Self::MAX_SIZE,
            );
            self.cigar = Cigar::new(len, len);
            self.len = len;
        }
    }
}

impl<const LOCAL: bool, const PREFIX_SUFFIX: bool, const LOCAL_PREFIX_SUFFIX: bool> Aligner
    for MatchAligner<LOCAL, PREFIX_SUFFIX, LOCAL_PREFIX_SUFFIX>
{
    fn align(
        &mut self,
        read: &[u8],
        pattern: &[u8],
        identity_threshold: f64,
        overlap_threshold: f64,
        prefix: bool,
    ) -> Option<(usize, usize, usize)> {
        let padding_len = if PREFIX_SUFFIX {
            ((1.0 - overlap_threshold).max(0.0) * (pattern.len() as f64)).ceil() as usize
        } else {
            0
        };

        self.resize_if_needed(pattern.len().max(read.len() + padding_len));
        self.read_vec.clear();

        if prefix {
            self.read_vec.extend((0..padding_len).map(|_| b'X'));
            self.read_vec.extend_from_slice(read);

            self.read_padded
                .set_bytes_rev::<NucMatrix>(&self.read_vec, Self::MAX_SIZE);
            self.pattern_padded
                .set_bytes_rev::<NucMatrix>(pattern, Self::MAX_SIZE);
        } else {
            self.read_vec.extend_from_slice(read);
            self.read_vec.extend((0..padding_len).map(|_| b'X'));

            self.read_padded
                .set_bytes::<NucMatrix>(&self.read_vec, Self::MAX_SIZE);
            self.pattern_padded
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
            &self.pattern_padded,
            &self.read_padded,
            &self.matrix,
            gaps,
            min_size..=Self::MAX_SIZE,
            pattern.len() as i32,
        );

        let res = self.block.res();
        self.block.trace().cigar_eq(
            &self.pattern_padded,
            &self.read_padded,
            res.query_idx,
            res.reference_idx,
            &mut self.cigar,
        );

        let mut matches = 0;
        let mut total = 0;

        self.cigar.reverse();
        let mut idx = res.reference_idx;

        for i in 0..self.cigar.len() {
            let OpLen { op, mut len } = self.cigar.get(i);

            match op {
                Operation::Eq => {
                    let prev_len = len;

                    if i == self.cigar.len() - 1 {
                        len -= idx.saturating_sub(read.len());
                    }

                    idx -= prev_len;
                    matches += len;
                }
                Operation::X => {
                    idx -= len;
                }
                Operation::D => {
                    idx -= len;
                }
                _ => (),
            }

            total += len;
        }

        let identity = (matches as f64) / (total as f64);
        let overlap = (matches as f64) / (pattern.len() as f64);

        if identity >= identity_threshold && overlap >= overlap_threshold {
            let start_idx = if prefix { read.len() - idx } else { idx };
            let end_idx = if prefix {
                read.len() - res.reference_idx.min(read.len())
            } else {
                res.reference_idx.min(read.len())
            };

            Some((matches, start_idx, end_idx))
        } else {
            None
        }
    }
}
