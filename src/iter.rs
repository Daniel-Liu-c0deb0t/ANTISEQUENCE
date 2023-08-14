use std::marker::{Send, Sync};
use std::ops::RangeBounds;
use std::sync::Arc;
use std::thread;

use crate::errors::*;
use crate::expr::*;
use crate::read::*;

pub mod cut_node;
pub use cut_node::*;

pub mod bernoulli_node;
pub use bernoulli_node::*;

pub mod time_node;
pub use time_node::*;

pub mod trim_node;
pub use trim_node::*;

pub mod count_node;
pub use count_node::*;

pub mod take_node;
pub use take_node::*;

pub mod set_node;
pub use set_node::*;

pub mod for_each_node;
pub use for_each_node::*;

pub mod retain_node;
pub use retain_node::*;

pub mod intersect_union_node;
pub use intersect_union_node::*;

pub mod fork_node;
pub use fork_node::*;

pub mod match_polyx_node;
pub use match_polyx_node::*;

pub mod match_regex_node;
pub use match_regex_node::*;

pub mod match_any_node;
pub use match_any_node::*;

pub mod collect_fastq_node;
pub use collect_fastq_node::*;

pub struct Graph {
    nodes: Vec<Arc<dyn GraphNode>>,
}

pub trait GraphNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)>;
    fn required_names(&self) -> &[LabelOrAttr];
    fn name(&self) -> &'static str;
}

impl Graph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add<G: GraphNode + 'static>(&mut self, node: G) -> Arc<G> {
        let a = Arc::new(node);
        let b = Arc::clone(&a);
        self.nodes.push(a);
        b
    }

    pub fn run(&self) -> Result<()> {
        loop {
            let (_, done) = self.run_one(None)?;
            if done {
                break;
            }
        }

        Ok(())
    }

    pub fn run_one(&self, mut curr: Option<Read>) -> Result<(Option<Read>, bool)> {
        for node in &self.nodes {
            if let Some(read) = &curr {
                if !read.has_names(node.required_names()) {
                    continue;
                }
            }

            let (c, done) = node.run(curr)?;
            curr = c;

            if done {
                return Ok((curr, done));
            }
            if curr.is_none() {
                break;
            }
        }

        Ok((curr, false))
    }
}

/*
/// Shared interface for all read iterators.
///
/// Many operations allow a select expression to be specified as the first parameter.
/// This ensures that the operation is only be applied on the selected reads.
pub trait Reads: Send + Sync {
    /// Run a `Reads` iterator until there are no more reads left.
    fn run(mut self) -> Result<()>
    where
        Self: Sized,
    {
        while !self.next_chunk()?.is_empty() {}
        self.finish()
    }

    /// Run a `Reads` iterator in parallel with multithreading.
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

    /// Run a `Reads` iterator and collect the resulting `Read`s into a `Vec`.
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

    /// Apply an arbitrary function on each read.
    #[must_use]
    fn for_each<F>(self, selector_expr: SelectorExpr, func: F) -> ForEachReads<Self, F>
    where
        F: Fn(&mut Read) + Send + Sync,
        Self: Sized,
    {
        ForEachReads::new(self, selector_expr, func)
    }

    /// Print each read to standard error.
    #[must_use]
    fn dbg(self, selector_expr: SelectorExpr) -> ForEachReads<Self, fn(&mut Read)>
    where
        Self: Sized,
    {
        ForEachReads::new(self, selector_expr, |read| eprintln!("{}", read))
    }

    /// Remove mappings with labels that start with `_` ("internal" mappings).
    #[must_use]
    fn remove_internal(self, selector_expr: SelectorExpr) -> ForEachReads<Self, fn(&mut Read)>
    where
        Self: Sized,
    {
        ForEachReads::new(self, selector_expr, |read| read.remove_internal())
    }

    /// Count the number of reads that are selected with each selector and apply an arbitrary
    /// function on the counts at the end.
    #[must_use]
    fn count<F>(self, selector_exprs: impl Into<Vec<SelectorExpr>>, func: F) -> CountReads<Self, F>
    where
        F: Fn(&[usize]) + Send + Sync,
        Self: Sized,
    {
        CountReads::new(self, selector_exprs.into(), func)
    }

    /// Check whether a mapping length is within the specified bounds.
    ///
    /// The transform expression must have one input mapping and one output mapping.
    ///
    /// Example `transform_expr`: `tr!(seq1.* -> seq1.*.in_bounds)`.
    /// This will set `seq1.*.in_bounds` to a boolean indicating whether the length of `seq1.*`
    /// is in the specified bounds.
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

    /// Set an attribute to true with some probability.
    ///
    /// This is deterministic, even with multithreading.
    #[must_use]
    fn bernoulli(
        self,
        selector_expr: SelectorExpr,
        attr: Attr,
        prob: f64,
        seed: u32,
    ) -> BernoulliReads<Self>
    where
        Self: Sized,
    {
        BernoulliReads::new(self, selector_expr, attr, prob, seed)
    }

    /// Cut a mapping at an index to create two new mappings.
    ///
    /// The transform expression must have one input mapping and two output mappings.
    ///
    /// Example `transform_expr`: `tr!(seq1.* -> seq1.left, seq1.right)`.
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

    /// Intersect two mapping intervals and create a new mapping of the intersection, if it is not empty.
    ///
    /// The transform expression must have two input mappings and one output mapping.
    ///
    /// Example `transform_expr`: `tr!(seq1.a, seq1.b -> seq1.c)`.
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

    /// Union two mapping intervals and create a new mapping of the union.
    ///
    /// If the two mapping intervals are disjoint, then the union will also contain the region
    /// between the two mapping intervals, which is not inside either mapping intervals.
    ///
    /// The transform expression must have two input mappings and one output mapping.
    ///
    /// Example `transform_expr`: `tr!(seq1.a, seq1.b -> seq1.c)`.
    #[must_use]
    fn union(self, selector_expr: SelectorExpr, transform_expr: TransformExpr) -> UnionReads<Self>
    where
        Self: Sized,
    {
        UnionReads::new(self, selector_expr, transform_expr)
    }

    /// Trim the mappings corresponding to the specified labels by modifying the underlying strings.
    ///
    /// When a mapping is trimmed, its length will be set to zero. All intersecting
    /// mappings will also be adjusted accordingly for the shortening.
    #[must_use]
    fn trim(self, selector_expr: SelectorExpr, labels: impl Into<Vec<Label>>) -> TrimReads<Self>
    where
        Self: Sized,
    {
        TrimReads::new(self, selector_expr, labels.into())
    }

    /// Set a label or attribute to the result of a format expression.
    ///
    /// After a label is set, its mapping and all other intersecting mappings will be adjusted accordingly
    /// for any shortening or lengthening.
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

    /// Match a regex pattern in a mapping.
    ///
    /// If named capture groups are used, then mappings are automatically created at the match
    /// locations, labeled by the names specified in the regex.
    ///
    /// The transform expression must have one input mapping and one output attribute.
    ///
    /// Example `transform_expr`: `tr!(seq1.* -> seq1.*.matched)`.
    /// This will match the regex pattern two `seq1.*` and set `seq1.*.matched` to a boolean
    /// indicating whether the regex matches.
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

    /// Match any one of multiple patterns in a mapping.
    ///
    /// Patterns are specified in YAML format:
    /// ```
    /// name: my_patterns
    /// patterns:
    ///   - pattern: AAAA
    ///     some_extra_data1: !String "all As"
    ///     some_extra_data2: !Bool true
    ///   - pattern: TTTT
    ///     some_extra_data1: !String "all Ts"
    ///     some_extra_data2: !Bool false
    /// ```
    ///
    /// Patterns can be arbitrary format expressions, so you can use any existing mappings or
    /// attributes as patterns.
    ///
    /// You can also include arbitrary extra attributes, like `some_extra_data1` and
    /// `some_extra_data2` in this example. The corresponding attributes for the matched pattern
    /// will be stored into the input mapping.
    ///
    /// The transform expression must have one input mapping and the number of output mappings is
    /// determined by the [`MatchType`].
    ///
    /// Example `transform_expr` for local-alignment-based pattern matching:
    /// `tr!(seq1.* -> seq1.before, seq1.aligned, seq1.after)`.
    /// The input mapping will get a new attribute (`seq1.*.my_patterns`) that is set to the pattern
    /// that is matched. If no pattern matches, then it will be set to false.
    /// Assuming pattern `AAAA` is matched, `seq1.*.some_extra_data1` will be set to `"all As"` and
    /// `seq1.*.some_extra_data2` will be set to `true`.
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

    /// Match a pattern in a mapping.
    ///
    /// The pattern can be an arbitrary format expression, so you can use any existing mappings or
    /// attributes as patterns.
    ///
    /// The transform expression must have one input mapping and the number of output mappings is
    /// determined by the [`MatchType`].
    ///
    /// Example `transform_expr` for local-alignment-based pattern matching:
    /// `tr!(seq1.* -> seq1.before, seq1.aligned, seq1.after)`.
    #[must_use]
    fn match_one(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        pattern: impl AsRef<str>,
        match_type: MatchType,
    ) -> MatchAnyReads<Self>
    where
        Self: Sized,
    {
        MatchAnyReads::new(
            self,
            selector_expr,
            transform_expr,
            Patterns::new(vec![FormatExpr::new(pattern.as_ref().as_bytes())
                .unwrap_or_else(|e| {
                    panic!("Error in parsing format expression for the match_one operation: {e}")
                })]),
            match_type,
        )
    }

    /// Match repeated characters from the left or right end of a mapping.
    ///
    /// The transform expression must have one input mapping and two output mappings.
    ///
    /// Example `transform_expr`: `tr!(seq1.* -> seq1.sequence, seq1.polya_tail)`.
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

    /// Output reads to a specified file.
    ///
    /// The file path is a format expression.
    ///
    /// Only read 1 is written out.
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

    /// Output paired-end reads to the specified files.
    ///
    /// The file paths are format expressions.
    ///
    /// Read 1 is written to `file_expr1` and read 2 is written to `file_expr2`.
    /// The reads will be interleaved if the files are the same.
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

    /// Retain only the reads that are selected and discard the rest.
    #[must_use]
    fn retain(self, selector_expr: SelectorExpr) -> RetainReads<Self>
    where
        Self: Sized,
    {
        RetainReads::new(self, selector_expr)
    }

    /// Take only the reads that have a record index inside the bounds.
    #[must_use]
    fn take<B>(self, bounds: B) -> TakeReads<Self, B>
    where
        B: RangeBounds<usize> + Send + Sync,
        Self: Sized,
    {
        TakeReads::new(self, bounds)
    }

    /// Create two read iterators by cloning each read.
    ///
    /// You must use the [`run!()`](crate::run!) or [`run_with_threads!()`](crate::run_with_threads!) macros to run all the forks.
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

    /// Compute the runtime (in seconds) of all operations before this in the iterator chain.
    ///
    /// The runtime is summed across all threads.
    ///
    /// The function `func` is called at the end with the runtime.
    #[must_use]
    fn time<F>(self, func: F) -> TimeReads<Self, F>
    where
        F: Fn(f64) + Send + Sync,
        Self: Sized,
    {
        TimeReads::new(self, func)
    }

    /// Box the read iterator by creating a `Box<dyn Reads>`.
    ///
    /// This allows iterators to be dynamically chained at runtime.
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

/// Run one or more `Reads` iterators until there are no more reads left.
///
/// This should be used to run iterators that are forked.
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
            fn check_type_and_drop<R: Reads>(_reads: R) {}
            check_type_and_drop(first);
        }
    };
    (@finish $first:expr, $($e:expr),*) => {
        {
            let mut first = $first;
            first.finish()
                .unwrap_or_else(|e| panic!("Error when running: {e}"));
            fn check_type_and_drop<R: Reads>(_reads: R) {}
            check_type_and_drop(first);
            run!(@finish $($e),*);
        }
    };
}

/// Run one or more `Reads` iterators in parallel with multithreading.
///
/// The first parameter is the number of threads to use.
///
/// This should be used to run iterators that are forked.
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
            fn check_type_and_drop<R: Reads>(_reads: R) {}
            check_type_and_drop(first);
        }
    };
    (@finish $first:expr, $($e:expr),*) => {
        {
            let mut first = $first;
            first.finish()
                .unwrap_or_else(|e| panic!("Error when running: {e}"));
            fn check_type_and_drop<R: Reads>(_reads: R) {}
            check_type_and_drop(first);
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
*/

pub use MatchType::*;
pub use Threshold::*;

/// Algorithm types for matching patterns.
///
/// For alignment-based algorithms, `sequence identity = matches / (matches + mismatches + insertions + deletions)`
/// and `overlap = matches / pattern_length`.
///
/// Insertions and deletions that are not part of the alignment are not included in the sequence
/// identity computation. This is important for local alignment, where the start and end of the
/// pattern can be excluded from the alignment, and prefix/suffix alignment, where the start/end
/// of the pattern can be excluded from the alignment (prefix/suffix "overhang").
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MatchType {
    /// Exact match.
    ///
    /// A match will result in one new mapping: the entire string.
    Exact,
    /// Exact prefix match.
    ///
    /// A match will result in two new mappings: the matched prefix and the rest of the string.
    ExactPrefix,
    /// Exact suffix match.
    ///
    /// A match will result in two new mappings: the rest of the string and the matched
    /// suffix.
    ExactSuffix,
    /// Exact match search.
    ///
    /// A match will result in three new mappings: everything before the exact match, the exact
    /// matching region, everything after the exact match.
    ExactSearch,
    /// Hamming-distance-based matching.
    ///
    /// Threshold is for the number of matching bases.
    ///
    /// A match will result in one new mapping: the entire string.
    Hamming(Threshold),
    /// Hamming-distance-based prefix matching.
    ///
    /// Threshold is for the number of matching bases.
    ///
    /// A match will result in two new mappings: the matched prefix and the rest of the
    /// string.
    HammingPrefix(Threshold),
    /// Hamming-distance-based suffix matching.
    ///
    /// Threshold is for the number of matching bases.
    ///
    /// A match will result in two new mappings: the rest of the string and the matched
    /// suffix.
    HammingSuffix(Threshold),
    /// Hamming-distance-based searching.
    ///
    /// Threshold is for the number of matching bases.
    ///
    /// A match will result in three new mappings: everything before the match, the matching
    /// region, and everything after the match.
    HammingSearch(Threshold),
    /// Global-alignment-based matching.
    ///
    /// Threshold is for the sequence identity.
    ///
    /// A match will result in one new mapping: the entire string.
    GlobalAln(f64),
    /// Local-alignment-based matching.
    ///
    /// A match will result in three new mappings: everything before the aligned region, the locally aligned
    /// region, and everything after the aligned region.
    LocalAln { identity: f64, overlap: f64 },
    /// Prefix-alignment-based matching.
    ///
    /// A match will result in two new mappings: the matched prefix and the rest of the
    /// string.
    PrefixAln { identity: f64, overlap: f64 },
    /// Suffix-alignment-based matching.
    ///
    /// A match will result in two new mappings: the rest of the string and the matched
    /// suffix.
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
            ExactSearch | HammingSearch(_) | LocalAln { .. } => 3,
        }
    }
}

/// Either a count or a fraction.
///
/// Typically used for specifying the similarity threshold when matching patterns.
/// The fraction is typically of the length of the pattern.
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
