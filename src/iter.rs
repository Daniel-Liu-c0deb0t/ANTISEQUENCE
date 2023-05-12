use std::ops::RangeBounds;
use std::thread;

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
    fn run(self, threads: usize) {
        assert!(threads >= 1);

        thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| while self.next_chunk().len() > 0 {});
            }
        });

        self.finish();
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
    fn count<F>(self, selector_exprs: impl AsRef<[SelectorExpr]>, func: F) -> CountReads<Self, F>
    where
        F: Fn(&[usize]) + std::marker::Sync,
    {
        let selector_exprs = selector_exprs.as_ref().to_owned();
        CountReads::new(self, selector_exprs, func)
    }

    #[must_use]
    fn length_in_bounds<B>(
        self,
        selector_expr: SelectorExpr,
        attr: Attr,
        bounds: B,
    ) -> LengthInBoundsReads<Self, B>
    where
        B: RangeBounds<usize> + std::marker::Sync,
    {
        LengthInBoundsReads::new(self, selector_expr, attr, bounds)
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
    fn trim(self, selector_expr: SelectorExpr, labels: impl AsRef<[Label]>) -> TrimReads<Self> {
        let labels = labels.as_ref().to_owned();
        TrimReads::new(self, selector_expr, labels)
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
            FormatExpr::new(format_expr.as_ref().as_bytes()),
        )
    }

    #[must_use]
    fn match_regex(
        self,
        selector_expr: SelectorExpr,
        attr: Attr,
        regex: impl AsRef<str>,
    ) -> MatchRegexReads<Self> {
        MatchRegexReads::new(self, selector_expr, attr, regex.as_ref())
    }

    #[must_use]
    fn match_any(
        self,
        selector_expr: SelectorExpr,
        label: Label,
        patterns_yaml: impl AsRef<str>,
        match_type: MatchType,
    ) -> MatchAnyReads<Self> {
        MatchAnyReads::new(
            self,
            selector_expr,
            label,
            Patterns::from_yaml(patterns_yaml.as_ref().as_bytes()),
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
            FormatExpr::new(file_expr.as_ref().as_bytes()),
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
            FormatExpr::new(file_expr1.as_ref().as_bytes()),
            FormatExpr::new(file_expr2.as_ref().as_bytes()),
        )
    }

    #[must_use]
    fn retain(self, selector_expr: impl AsRef<str>) -> RetainReads<Self> {
        RetainReads::new(self, SelectorExpr::new(selector_expr.as_ref().as_bytes()))
    }

    fn next_chunk(&self) -> Vec<Read>;

    fn finish(&self);
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
    GlobalAln(Threshold),
    LocalAln(Threshold),
    PrefixAln(Threshold),
    SuffixAln(Threshold),
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
