use std::ops::RangeBounds;
use std::thread;

use crate::errors::*;
use crate::expr::*;
use crate::patterns::*;
use crate::read::*;

pub mod trim_reads;
use trim_reads::*;

pub mod collect_fastq_reads;
use collect_fastq_reads::*;

pub mod for_each_reads;
use for_each_reads::*;

pub mod cut_reads;
use cut_reads::*;

pub mod set_reads;
use set_reads::*;

pub mod length_in_bounds_reads;
use length_in_bounds_reads::*;

pub mod retain_reads;
use retain_reads::*;

pub mod match_regex_reads;
use match_regex_reads::*;

pub mod match_any_reads;
use match_any_reads::*;

pub mod count_reads;
use count_reads::*;

pub trait Reads: Sized + std::marker::Sync {
    fn run(self) -> Result<()> {
        while !self.next_chunk()?.is_empty() {}
        self.finish()
    }

    fn run_with_threads(self, threads: usize) {
        assert!(threads >= 1, "Number of threads must be greater than zero");

        thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| {
                    while !self
                        .next_chunk()
                        .unwrap_or_else(|e| panic!("Error when running: {e}"))
                        .is_empty()
                    {}
                });
            }
        });

        self.finish()
            .unwrap_or_else(|e| panic!("Error when running: {e}"));
    }

    fn run_collect_reads(self) -> Result<Vec<Read>> {
        let mut res = Vec::new();

        loop {
            let reads = self.next_chunk()?;

            if reads.is_empty() {
                break;
            }

            res.extend(reads);
        }

        self.finish()?;
        Ok(res)
    }

    #[must_use]
    fn for_each<F>(self, selector_expr: SelectorExpr, func: F) -> ForEachReads<Self, F>
    where
        F: Fn(&mut Read) + std::marker::Sync,
    {
        ForEachReads::new(self, selector_expr, func)
    }

    #[must_use]
    fn dbg(self, selector_expr: SelectorExpr) -> ForEachReads<Self, fn(&mut Read)> {
        ForEachReads::new(self, selector_expr, |read| eprintln!("{}", read))
    }

    #[must_use]
    fn count<F>(self, selector_exprs: impl Into<Vec<SelectorExpr>>, func: F) -> CountReads<Self, F>
    where
        F: Fn(&[usize]) + std::marker::Sync,
    {
        CountReads::new(self, selector_exprs.into(), func)
    }

    #[must_use]
    fn length_in_bounds<B>(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        bounds: B,
    ) -> LengthInBoundsReads<Self, B>
    where
        B: RangeBounds<usize> + std::marker::Sync,
    {
        LengthInBoundsReads::new(self, selector_expr, transform_expr, bounds)
    }

    #[must_use]
    fn cut(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        cut_idx: EndIdx,
    ) -> CutReads<Self> {
        CutReads::new(self, selector_expr, transform_expr, cut_idx)
    }

    #[must_use]
    fn trim(self, selector_expr: SelectorExpr, labels: impl Into<Vec<Label>>) -> TrimReads<Self> {
        TrimReads::new(self, selector_expr, labels.into())
    }

    #[must_use]
    fn set(
        self,
        selector_expr: SelectorExpr,
        label_or_attr: impl Into<LabelOrAttr>,
        format_expr: impl AsRef<str>,
    ) -> SetReads<Self> {
        SetReads::new(
            self,
            selector_expr,
            label_or_attr.into(),
            FormatExpr::new(format_expr.as_ref().as_bytes()).unwrap_or_else(|e| {
                panic!("Error in parsing format expression for the set operation: {e}")
            }),
        )
    }

    #[must_use]
    fn match_regex(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        regex: impl AsRef<str>,
    ) -> MatchRegexReads<Self> {
        MatchRegexReads::new(self, selector_expr, transform_expr, regex.as_ref())
    }

    #[must_use]
    fn match_any(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        patterns_yaml: impl AsRef<str>,
        match_type: MatchType,
    ) -> MatchAnyReads<Self> {
        MatchAnyReads::new(
            self,
            selector_expr,
            transform_expr,
            Patterns::from_yaml(patterns_yaml.as_ref().as_bytes())
                .unwrap_or_else(|e| panic!("Error in parsing patterns: {e}")),
            match_type,
        )
    }

    #[must_use]
    fn collect_fastq1(
        self,
        selector_expr: SelectorExpr,
        file_expr: impl AsRef<str>,
    ) -> CollectFastqReads<Self> {
        CollectFastqReads::new1(
            self,
            selector_expr,
            FormatExpr::new(file_expr.as_ref().as_bytes()).unwrap_or_else(|e| {
                panic!("Error in parsing format expression for the collect_fastq1 operation: {e}")
            }),
        )
    }

    #[must_use]
    fn collect_fastq2(
        self,
        selector_expr: SelectorExpr,
        file_expr1: impl AsRef<str>,
        file_expr2: impl AsRef<str>,
    ) -> CollectFastqReads<Self> {
        CollectFastqReads::new2(
            self,
            selector_expr,
            FormatExpr::new(file_expr1.as_ref().as_bytes()).unwrap_or_else(|e| {
                panic!("Error in parsing format expression for the collect_fastq2 operation: {e}")
            }),
            FormatExpr::new(file_expr2.as_ref().as_bytes()).unwrap_or_else(|e| {
                panic!("Error in parsing format expression for the collect_fastq2 operation: {e}")
            }),
        )
    }

    #[must_use]
    fn retain(self, selector_expr: impl AsRef<str>) -> RetainReads<Self> {
        RetainReads::new(
            self,
            SelectorExpr::new(selector_expr.as_ref().as_bytes()).unwrap_or_else(|e| {
                panic!("Error in parsing selector expression for the retain operation: {e}")
            }),
        )
    }

    fn next_chunk(&self) -> Result<Vec<Read>>;

    fn finish(&self) -> Result<()>;
}

pub use MatchType::*;
pub use Threshold::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MatchType {
    Exact,
    ExactPrefix,
    ExactSuffix,
    Hamming(Threshold),
    HammingPrefix(Threshold),
    HammingSuffix(Threshold),
    GlobalAln(f64),
    LocalAln { identity: f64, overlap: f64 },
    PrefixAln { identity: f64, overlap: f64 },
    SuffixAln { identity: f64, overlap: f64 },
}

impl MatchType {
    pub fn num_mappings(&self) -> usize {
        use MatchType::*;
        match self {
            Exact | Hamming(_) | GlobalAln(_) => 1,
            ExactPrefix
            | ExactSuffix
            | HammingPrefix(_)
            | HammingSuffix(_)
            | PrefixAln { .. }
            | SuffixAln { .. } => 2,
            LocalAln { .. } => 3,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Threshold {
    Count(usize),
    Frac(f64),
}

impl Threshold {
    pub fn get(&self, len: usize) -> usize {
        use Threshold::*;
        match self {
            Count(c) => *c,
            Frac(f) => (*f * (len as f64)) as usize,
        }
    }
}
