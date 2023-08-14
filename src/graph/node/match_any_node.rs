use block_aligner::{cigar::*, scan_block::*, scores::*};

use memchr::memmem;

use thread_local::*;

use std::cell::RefCell;
use std::marker::Send;

use crate::graph::*;
use crate::Patterns;

pub struct MatchAnyNode {
    required_names: Vec<LabelOrAttr>,
    label: Label,
    new_labels: [Option<Label>; 3],
    patterns: Patterns,
    match_type: MatchType,
    aligner: ThreadLocal<Option<RefCell<Box<dyn Aligner + Send>>>>,
}

impl MatchAnyNode {
    const NAME: &'static str = "matching any patterns";

    pub fn new(
        transform_expr: TransformExpr,
        patterns: Patterns,
        match_type: MatchType,
    ) -> Self {
        let mut new_labels = [None, None, None];

        transform_expr.check_size(1, match_type.num_mappings(), Self::NAME);
        for i in 0..match_type.num_mappings() {
            new_labels[i] = transform_expr.after_label(i, Self::NAME);
        }
        transform_expr.check_same_str_type(Self::NAME);

        Self {
            required_names: vec![transform_expr.before(0).into()],
            label: transform_expr.before(0),
            new_labels,
            patterns,
            match_type,
            aligner: ThreadLocal::new(),
        }
    }
}

impl GraphNode for MatchAnyNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

        let string = read
            .substring(self.label.str_type, self.label.label)
            .map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: Self::NAME,
            })?;

        let aligner_cell = self.aligner.get_or(|| {
            match self.match_type {
                MatchType::GlobalAln(_) => {
                    Some(RefCell::new(Box::new(GlobalLocalAligner::<false>::new(string.len() * 2))))
                }
                MatchType::LocalAln { .. } => {
                    Some(RefCell::new(Box::new(GlobalLocalAligner::<true>::new(string.len() * 2))))
                }
                MatchType::PrefixAln { .. } => {
                    Some(RefCell::new(Box::new(PrefixSuffixAligner::<true>::new(string.len() * 2))))
                }
                MatchType::SuffixAln { .. } => {
                    Some(RefCell::new(Box::new(PrefixSuffixAligner::<false>::new(string.len() * 2))))
                }
                _ => None,
            }
        });

        let mut max_matches = 0;
        let mut max_pattern = None;
        let mut max_cut_pos1 = 0;
        let mut max_cut_pos2 = 0;

        for pattern in self.patterns.patterns() {
            let pattern_str_cow =
                pattern
                    .get(&read)
                    .map_err(|e| Error::NameError {
                        source: e,
                        read: read.clone(),
                        context: Self::NAME,
                    })?;
            let pattern_str: &[u8] = &pattern_str_cow;
            let pattern_len = pattern_str.len();

            if max_matches >= pattern_len {
                continue;
            }

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
                    if pattern_len <= string.len() && &string[..pattern_len] == pattern_str {
                        Some((pattern_len, pattern_len, 0))
                    } else {
                        None
                    }
                }
                ExactSuffix => {
                    if pattern_len <= string.len()
                        && &string[string.len() - pattern_len..] == pattern_str
                    {
                        Some((pattern_len, string.len() - pattern_len, 0))
                    } else {
                        None
                    }
                }
                ExactSearch => memmem::find(string, pattern_str)
                    .map(|i| (pattern_len, i, i + pattern_len)),
                Hamming(t) => {
                    let t = t.get(pattern_len);
                    hamming(string, pattern_str, t).map(|m| (m, pattern_len, 0))
                }
                HammingPrefix(t) => {
                    if pattern_len <= string.len() {
                        let t = t.get(pattern_len);
                        hamming(&string[..pattern_len], pattern_str, t)
                            .map(|m| (m, pattern_len, 0))
                    } else {
                        None
                    }
                }
                HammingSuffix(t) => {
                    if pattern_len <= string.len() {
                        let t = t.get(pattern_len);
                        hamming(&string[string.len() - pattern_len..], pattern_str, t)
                            .map(|m| (m, string.len() - pattern_len, 0))
                    } else {
                        None
                    }
                }
                HammingSearch(t) => {
                    let t = t.get(pattern_len);
                    hamming_search(string, pattern_str, t)
                }
                GlobalAln(identity) => aligner_cell
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .align(string, pattern_str, identity, identity)
                    .map(|(m, _, end_idx)| (m, end_idx, 0)),
                LocalAln { identity, overlap } => {
                    aligner_cell
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .align(string, pattern_str, identity, overlap)
                }
                PrefixAln { identity, overlap } => {
                    let additional =
                        ((1.0 - identity).max(0.0) * (pattern_len as f64)).ceil() as usize;
                    let len = string.len().min(pattern_len + additional);
                    aligner_cell
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .align(&string[..len], pattern_str, identity, overlap)
                        .map(|(m, _, end_idx)| (m, end_idx, 0))
                }
                SuffixAln { identity, overlap } => {
                    let additional =
                        ((1.0 - identity).max(0.0) * (pattern_len as f64)).ceil() as usize;
                    let len = string.len().min(pattern_len + additional);
                    aligner_cell
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .align(
                            &string[string.len() - len..],
                            pattern_str,
                            identity,
                            overlap,
                        )
                        .map(|(m, start_idx, _)| (m, string.len() - len + start_idx, 0))
                }
            };

            if let Some((matches, cut_pos1, cut_pos2)) = matches {
                if matches > max_matches {
                    max_matches = matches;
                    max_pattern = Some((pattern_str_cow, pattern.attrs()));
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
            if let Some(pattern_name) = self.patterns.pattern_name() {
                *mapping.data_mut(pattern_name) = Data::Bytes(pattern_str.into_owned());
            }

            for (&attr, data) in self.patterns.attr_names().iter().zip(pattern_attrs) {
                *mapping.data_mut(attr) = data.clone();
            }

            match self.match_type.num_mappings() {
                1 => {
                    let start = mapping.start;
                    let str_mappings = read.str_mappings_mut(self.label.str_type).unwrap();
                    str_mappings.add_mapping(
                        self.new_labels[0].as_ref().map(|l| l.label),
                        start,
                        max_cut_pos1,
                    );
                }
                2 => {
                    read.cut(
                        self.label.str_type,
                        self.label.label,
                        self.new_labels[0].as_ref().map(|l| l.label),
                        self.new_labels[1].as_ref().map(|l| l.label),
                        LeftEnd(max_cut_pos1),
                    )
                    .unwrap_or_else(|e| panic!("Error {}: {e}", Self::NAME));
                }
                3 => {
                    let offset = mapping.start;
                    let mapping_len = mapping.len;

                    let str_mappings = read.str_mappings_mut(self.label.str_type).unwrap();
                    str_mappings.add_mapping(
                        self.new_labels[0].as_ref().map(|l| l.label),
                        offset,
                        max_cut_pos1,
                    );
                    str_mappings.add_mapping(
                        self.new_labels[1].as_ref().map(|l| l.label),
                        offset + max_cut_pos1,
                        max_cut_pos2 - max_cut_pos1,
                    );
                    str_mappings.add_mapping(
                        self.new_labels[2].as_ref().map(|l| l.label),
                        offset + max_cut_pos2,
                        mapping_len - max_cut_pos2,
                    );
                }
                _ => unreachable!(),
            }
        } else {
            if let Some(pattern_name) = self.patterns.pattern_name() {
                *mapping.data_mut(pattern_name) = Data::Bool(false);
            }
        }

        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &self.required_names
    }

    fn name(&self) -> &'static str {
        Self::NAME
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

fn hamming_search(a: &[u8], b: &[u8], threshold: usize) -> Option<(usize, usize, usize)> {
    let mut best_match = None;

    for (i, w) in a.windows(b.len()).enumerate() {
        if let Some(matches) = hamming(w, b, threshold) {
            if let Some((best_matches, _, _)) = best_match {
                if matches <= best_matches {
                    continue;
                }
            }

            best_match = Some((matches, i, i + b.len()));
        }
    }

    best_match
}

trait Aligner {
    fn align(
        &mut self,
        read: &[u8],
        pattern: &[u8],
        identity_threshold: f64,
        overlap_threshold: f64,
    ) -> Option<(usize, usize, usize)>;
}

struct GlobalLocalAligner<const LOCAL: bool> {
    read_padded: PaddedBytes,
    pattern_padded: PaddedBytes,
    matrix: NucMatrix,
    // always store trace
    block: Block<true, LOCAL, LOCAL, false>,
    cigar: Cigar,
    len: usize,
}

impl<const LOCAL: bool> GlobalLocalAligner<LOCAL> {
    const MIN_SIZE: usize = 32;
    const MAX_SIZE: usize = 512;
    const GAPS: Gaps = Gaps {
        open: -2,
        extend: -1,
    };

    pub fn new(len: usize) -> Self {
        let read_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
        let pattern_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
        let matrix = NucMatrix::new_simple(1, -1);

        let block = Block::<true, LOCAL, LOCAL, false>::new(len, len, Self::MAX_SIZE);
        let cigar = Cigar::new(len, len);

        Self {
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
            self.block = Block::<true, LOCAL, LOCAL, false>::new(len, len, Self::MAX_SIZE);
            self.cigar = Cigar::new(len, len);
            self.len = len;
        }
    }
}

unsafe impl<const LOCAL: bool> Send for GlobalLocalAligner<LOCAL> {}

impl<const LOCAL: bool> Aligner for GlobalLocalAligner<LOCAL> {
    fn align(
        &mut self,
        read: &[u8],
        pattern: &[u8],
        identity_threshold: f64,
        overlap_threshold: f64,
    ) -> Option<(usize, usize, usize)> {
        self.resize_if_needed(pattern.len().max(read.len()));

        let max_size = pattern
            .len()
            .min(read.len())
            .next_power_of_two()
            .min(Self::MAX_SIZE);

        self.read_padded.set_bytes::<NucMatrix>(read, max_size);
        self.pattern_padded
            .set_bytes::<NucMatrix>(pattern, max_size);

        let min_size = if LOCAL { max_size } else { Self::MIN_SIZE };

        self.block.align(
            &self.pattern_padded,
            &self.read_padded,
            &self.matrix,
            Self::GAPS,
            min_size..=max_size,
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
        let mut read_start_idx = res.reference_idx;

        for i in 0..self.cigar.len() {
            let OpLen { op, len } = self.cigar.get(i);

            match op {
                Operation::Eq => {
                    read_start_idx -= len;
                    matches += len;
                }
                Operation::X => {
                    read_start_idx -= len;
                }
                Operation::D => {
                    read_start_idx -= len;
                }
                _ => (),
            }

            total += len;
        }

        let identity = (matches as f64) / (total as f64);
        let overlap = (matches as f64) / (pattern.len() as f64);

        if identity >= identity_threshold && overlap >= overlap_threshold {
            Some((matches, read_start_idx, res.reference_idx))
        } else {
            None
        }
    }
}

struct PrefixSuffixAligner<const PREFIX: bool> {
    read_padded: PaddedBytes,
    pattern_padded: PaddedBytes,
    matrix: NucMatrix,
    // always store trace
    block1: Block<true, true, false, true>,  // X-drop
    block2: Block<true, false, false, true>, // no X-drop
    cigar: Cigar,
    len: usize,
}

impl<const PREFIX: bool> PrefixSuffixAligner<PREFIX> {
    const MAX_SIZE: usize = 512;
    const GAPS: Gaps = Gaps {
        open: -2,
        extend: -1,
    };

    pub fn new(len: usize) -> Self {
        let read_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
        let pattern_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
        let matrix = NucMatrix::new_simple(1, -1);

        let block1 = Block::<true, true, false, true>::new(len, len, Self::MAX_SIZE);
        let block2 = Block::<true, false, false, true>::new(len, len, Self::MAX_SIZE);
        let cigar = Cigar::new(len, len);

        Self {
            read_padded,
            pattern_padded,
            matrix,
            block1,
            block2,
            cigar,
            len,
        }
    }

    fn resize_if_needed(&mut self, len: usize) {
        if len > self.len {
            self.read_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
            self.pattern_padded = PaddedBytes::new::<NucMatrix>(len, Self::MAX_SIZE);
            self.block1 = Block::<true, true, false, true>::new(len, len, Self::MAX_SIZE);
            self.block2 = Block::<true, false, false, true>::new(len, len, Self::MAX_SIZE);
            self.cigar = Cigar::new(len, len);
            self.len = len;
        }
    }
}

unsafe impl<const PREFIX: bool> Send for PrefixSuffixAligner<PREFIX> {}

impl<const PREFIX: bool> Aligner for PrefixSuffixAligner<PREFIX> {
    fn align(
        &mut self,
        read: &[u8],
        pattern: &[u8],
        identity_threshold: f64,
        overlap_threshold: f64,
    ) -> Option<(usize, usize, usize)> {
        self.resize_if_needed(pattern.len().max(read.len()));

        let max_size = pattern
            .len()
            .min(read.len())
            .next_power_of_two()
            .min(Self::MAX_SIZE);

        if PREFIX {
            // reverse sequences to convert to aligning suffix
            self.read_padded.set_bytes_rev::<NucMatrix>(read, max_size);
            self.pattern_padded
                .set_bytes_rev::<NucMatrix>(pattern, max_size);
        } else {
            self.read_padded.set_bytes::<NucMatrix>(read, max_size);
            self.pattern_padded
                .set_bytes::<NucMatrix>(pattern, max_size);
        }

        // first align to get where the pattern starts in the read
        // note that the start gaps in the pattern are free and the alignment
        // can end whenever due to X-drop
        self.block1.align(
            &self.pattern_padded,
            &self.read_padded,
            &self.matrix,
            Self::GAPS,
            max_size..=max_size,
            pattern.len() as i32,
        );

        let res = self.block1.res();
        self.block1.trace().cigar_eq(
            &self.pattern_padded,
            &self.read_padded,
            res.query_idx,
            res.reference_idx,
            &mut self.cigar,
        );

        // use traceback to compute where the alignment started
        let mut read_start_idx = res.reference_idx;
        for i in 0..self.cigar.len() {
            let OpLen { op, len } = self.cigar.get(i);
            match op {
                Operation::Eq | Operation::X | Operation::D => read_start_idx -= len,
                _ => (),
            }
        }

        // skip second alignment if first alignment reaches the end of the read
        if res.reference_idx < read.len() {
            // get the overlapping prefix/suffix region
            if PREFIX {
                self.read_padded
                    .set_bytes::<NucMatrix>(&read[..read.len() - read_start_idx], max_size);
                self.pattern_padded
                    .set_bytes::<NucMatrix>(pattern, max_size);
            } else {
                self.read_padded
                    .set_bytes_rev::<NucMatrix>(&read[read_start_idx..], max_size);
                self.pattern_padded
                    .set_bytes_rev::<NucMatrix>(pattern, max_size);
            }

            // align again with read and pattern switched and reversed so that end gaps in the read
            // are free and the alignment ends at read_start_idx and spans the entire pattern
            self.block2.align(
                &self.read_padded,
                &self.pattern_padded,
                &self.matrix,
                Self::GAPS,
                max_size..=max_size,
                pattern.len() as i32,
            );

            let res = self.block2.res();
            self.block2.trace().cigar_eq(
                &self.read_padded,
                &self.pattern_padded,
                res.query_idx,
                res.reference_idx,
                &mut self.cigar,
            );
        }

        // count matches and total columns for calculating identity and overlap
        let mut matches = 0;
        let mut total = 0;

        for i in 0..self.cigar.len() {
            let OpLen { op, len } = self.cigar.get(i);
            if op == Operation::Eq {
                matches += len;
            }
            total += len;
        }

        let identity = (matches as f64) / (total as f64);
        let overlap = (matches as f64) / (pattern.len() as f64);

        if identity >= identity_threshold && overlap >= overlap_threshold {
            let start_idx = if PREFIX { 0 } else { read_start_idx };
            let end_idx = if PREFIX {
                read.len() - read_start_idx
            } else {
                read.len()
            };

            Some((matches, start_idx, end_idx))
        } else {
            None
        }
    }
}
