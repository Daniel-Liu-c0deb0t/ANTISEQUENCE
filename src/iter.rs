use std::marker::{Send, Sync};
use std::ops::RangeBounds;
use std::sync::Arc;
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

pub mod bernoulli_reads;
use bernoulli_reads::*;

pub mod take_reads;
use take_reads::*;

pub mod match_polyx_reads;
use match_polyx_reads::*;

pub mod intersect_union_reads;
use intersect_union_reads::*;

pub mod fork_reads;
use fork_reads::*;

pub mod time_reads;
use time_reads::*;

pub trait Reads: Send + Sync {
    fn run(mut self) -> Result<()>
    where
        Self: Sized,
    {
        while !self.next_chunk()?.is_empty() {}
        self.finish()
    }

    fn run_with_threads(mut self, threads: usize)
    where
        Self: Sized,
    {
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

    fn run_collect_reads(mut self) -> Result<Vec<Read>>
    where
        Self: Sized,
    {
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
        F: Fn(&mut Read) + Send + Sync,
        Self: Sized,
    {
        ForEachReads::new(self, selector_expr, func)
    }

    #[must_use]
    fn dbg(self, selector_expr: SelectorExpr) -> ForEachReads<Self, fn(&mut Read)>
    where
        Self: Sized,
    {
        ForEachReads::new(self, selector_expr, |read| eprintln!("{}", read))
    }

    #[must_use]
    fn count<F>(self, selector_exprs: impl Into<Vec<SelectorExpr>>, func: F) -> CountReads<Self, F>
    where
        F: Fn(&[usize]) + Send + Sync,
        Self: Sized,
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
        B: RangeBounds<usize> + Send + Sync,
        Self: Sized,
    {
        LengthInBoundsReads::new(self, selector_expr, transform_expr, bounds)
    }

    #[must_use]
    fn bernoulli(
        self,
        selector_expr: SelectorExpr,
        attr: Attr,
        prob: f64,
        seed: u64,
    ) -> BernoulliReads<Self>
    where
        Self: Sized,
    {
        BernoulliReads::new(self, selector_expr, attr, prob, seed)
    }

    #[must_use]
    fn cut(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        cut_idx: EndIdx,
    ) -> CutReads<Self>
    where
        Self: Sized,
    {
        CutReads::new(self, selector_expr, transform_expr, cut_idx)
    }

    #[must_use]
    fn intersect(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
    ) -> IntersectReads<Self>
    where
        Self: Sized,
    {
        IntersectReads::new(self, selector_expr, transform_expr)
    }

    #[must_use]
    fn union(self, selector_expr: SelectorExpr, transform_expr: TransformExpr) -> UnionReads<Self>
    where
        Self: Sized,
    {
        UnionReads::new(self, selector_expr, transform_expr)
    }

    #[must_use]
    fn trim(self, selector_expr: SelectorExpr, labels: impl Into<Vec<Label>>) -> TrimReads<Self>
    where
        Self: Sized,
    {
        TrimReads::new(self, selector_expr, labels.into())
    }

    #[must_use]
    fn set(
        self,
        selector_expr: SelectorExpr,
        label_or_attr: impl Into<LabelOrAttr>,
        format_expr: impl AsRef<str>,
    ) -> SetReads<Self>
    where
        Self: Sized,
    {
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
    ) -> MatchRegexReads<Self>
    where
        Self: Sized,
    {
        MatchRegexReads::new(self, selector_expr, transform_expr, regex.as_ref())
    }

    #[must_use]
    fn match_any(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        patterns_yaml: impl AsRef<str>,
        match_type: MatchType,
    ) -> MatchAnyReads<Self>
    where
        Self: Sized,
    {
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
    fn match_polyx(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        x: char,
        end: End,
        identity: f64,
    ) -> MatchPolyXReads<Self>
    where
        Self: Sized,
    {
        MatchPolyXReads::new(self, selector_expr, transform_expr, x as u8, end, identity)
    }

    #[must_use]
    fn collect_fastq1(
        self,
        selector_expr: SelectorExpr,
        file_expr: impl AsRef<str>,
    ) -> CollectFastqReads<Self>
    where
        Self: Sized,
    {
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
    ) -> CollectFastqReads<Self>
    where
        Self: Sized,
    {
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
    fn retain(self, selector_expr: SelectorExpr) -> RetainReads<Self>
    where
        Self: Sized,
    {
        RetainReads::new(self, selector_expr)
    }

    #[must_use]
    fn take<B>(self, bounds: B) -> TakeReads<Self, B>
    where
        B: RangeBounds<usize> + Send + Sync,
        Self: Sized,
    {
        TakeReads::new(self, bounds)
    }

    #[must_use]
    fn fork(self) -> (ForkReads<Self>, ForkReads<Self>)
    where
        Self: Sized,
    {
        let reads = Arc::new(self);
        let buf = Arc::new(ForkBuf::new());
        let left = ForkReads::new(Arc::clone(&reads), Arc::clone(&buf));
        let right = ForkReads::new(reads, buf);
        (left, right)
    }

    #[must_use]
    fn time<F>(self, func: F) -> TimeReads<Self, F>
    where
        F: Fn(f64) + Send + Sync,
        Self: Sized,
    {
        TimeReads::new(self, func)
    }

    #[must_use]
    fn boxed(self) -> Box<dyn Reads>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    fn next_chunk(&self) -> Result<Vec<Read>>;

    fn finish(&mut self) -> Result<()>;
}

#[macro_export]
macro_rules! run {
    ($($e:expr),+ $(,)*) => {
        {
            let mut done = false;

            while !done {
                done = run!(@next_chunk $($e),+);
            }

            run!(@finish $($e),+);
        }
    };
    (@next_chunk $first:expr) => {
        {
            $first.next_chunk()
                .unwrap_or_else(|e| panic!("Error when running: {e}")).is_empty()
        }
    };
    (@next_chunk $first:expr, $($e:expr),*) => {
        {
            let empty = $first.next_chunk()
                .unwrap_or_else(|e| panic!("Error when running: {e}")).is_empty();
            empty & run!(@next_chunk $($e),*)
        }
    };
    (@finish $first:expr) => {
        {
            let mut first = $first;
            first.finish()
                .unwrap_or_else(|e| panic!("Error when running: {e}"));
        }
    };
    (@finish $first:expr, $($e:expr),*) => {
        {
            let mut first = $first;
            first.finish()
                .unwrap_or_else(|e| panic!("Error when running: {e}"));
            drop(first);
            run!(@finish $($e),*);
        }
    };
}

#[macro_export]
macro_rules! run_with_threads {
    ($threads:expr, $($e:expr),+ $(,)*) => {
        {
            assert!($threads >= 1, "Number of threads must be greater than zero");

            thread::scope(|s| {
                for _ in 0..$threads {
                    s.spawn(|| {
                        let mut done = false;

                        while !done {
                            done = run_with_threads!(@next_chunk $($e),+);
                        }
                    });
                }
            });

            run_with_threads!(@finish $($e),+);
        }
    };
    (@next_chunk $first:expr) => {
        {
            $first.next_chunk()
                .unwrap_or_else(|e| panic!("Error when running: {e}")).is_empty()
        }
    };
    (@next_chunk $first:expr, $($e:expr),*) => {
        {
            let empty = $first.next_chunk()
                .unwrap_or_else(|e| panic!("Error when running: {e}")).is_empty();
            empty & run_with_threads!(@next_chunk $($e),*)
        }
    };
    (@finish $first:expr) => {
        {
            let mut first = $first;
            first.finish()
                .unwrap_or_else(|e| panic!("Error when running: {e}"));
        }
    };
    (@finish $first:expr, $($e:expr),*) => {
        {
            let mut first = $first;
            first.finish()
                .unwrap_or_else(|e| panic!("Error when running: {e}"));
            drop(first);
            run_with_threads!(@finish $($e),*);
        }
    };
}

impl<R: Reads + ?Sized> Reads for Box<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        (**self).next_chunk()
    }

    fn finish(&mut self) -> Result<()> {
        (**self).finish()
    }
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
