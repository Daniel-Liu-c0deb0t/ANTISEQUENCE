use std::marker::{Send, Sync};
use std::ops::RangeBounds;
use std::sync::Arc;
use std::thread;

use crate::errors::*;
use crate::expr::*;
use crate::read::*;

pub mod node;

/// Computation graph of read operations, where each operation is a node.
pub struct Graph {
    nodes: Vec<Arc<dyn GraphNode>>,
}

pub trait GraphNode: Send + Sync {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)>;
    fn required_names(&self) -> &[LabelOrAttr];
    fn name(&self) -> &'static str;
}

impl Graph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add a read operation node to the graph and return the node.
    pub fn add<G: GraphNode + 'static>(&mut self, node: G) -> Arc<G> {
        let a = Arc::new(node);
        let b = Arc::clone(&a);
        self.nodes.push(a);
        b
    }

    /// Run a graph until all reads processed.
    pub fn run(&self) -> Result<()> {
        loop {
            let (_, done) = self.run_one(None)?;
            if done {
                break;
            }
        }

        Ok(())
    }

    /// Run a graph in parallel (multithreading) until all reads processed.
    pub fn run_with_threads(&self, threads: usize) {
        assert!(threads >= 1, "Number of threads must be greater than zero");

        thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| self.run().unwrap_or_else(|e| panic!("{e}")));
            }
        });
    }

    /// Run a single read through the graph.
    ///
    /// Returns an additional boolean indicating whether the graph is done executing.
    /// If the required label or attribute names for an operation are not available,
    /// the the operation is skipped.
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

    /// Try running a single read through the graph.
    ///
    /// Returns two booleans: the first one is whether the read has "failed" (does not have
    /// a required label or attribute name) and the second one is whether the graph is done
    /// executing.
    pub fn try_run_one(&self, mut curr: Option<Read>) -> Result<(Option<Read>, bool, bool)> {
        for node in &self.nodes {
            if let Some(read) = &curr {
                if !read.has_names(node.required_names()) {
                    return Ok((curr, true, false));
                }
            }

            let (c, done) = node.run(curr)?;
            curr = c;

            if done {
                return Ok((curr, false, done));
            }
            if curr.is_none() {
                break;
            }
        }

        Ok((curr, false, false))
    }
}

// TODO: update docs
/*
    /// Apply an arbitrary function on each read.
    fn for_each<F>(self, selector_expr: SelectorExpr, func: F) -> ForEachReads<Self, F>

    /// Print each read to standard error.
    fn dbg(self, selector_expr: SelectorExpr) -> ForEachReads<Self, fn(&mut Read)>

    /// Remove mappings with labels that start with `_` ("internal" mappings).
    fn remove_internal(self, selector_expr: SelectorExpr) -> ForEachReads<Self, fn(&mut Read)>

    /// Count the number of reads that are selected with each selector and apply an arbitrary
    /// function on the counts at the end.
    fn count<F>(self, selector_exprs: impl Into<Vec<SelectorExpr>>, func: F) -> CountReads<Self, F>

    /// Check whether a mapping length is within the specified bounds.
    ///
    /// The transform expression must have one input mapping and one output mapping.
    ///
    /// Example `transform_expr`: `tr!(seq1.* -> seq1.*.in_bounds)`.
    /// This will set `seq1.*.in_bounds` to a boolean indicating whether the length of `seq1.*`
    /// is in the specified bounds.
    fn length_in_bounds<B>(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        bounds: B,
    ) -> LengthInBoundsReads<Self, B>

    /// Set an attribute to true with some probability.
    ///
    /// This is deterministic, even with multithreading.
    fn bernoulli(
        self,
        selector_expr: SelectorExpr,
        attr: Attr,
        prob: f64,
        seed: u32,
    ) -> BernoulliReads<Self>

    /// Cut a mapping at an index to create two new mappings.
    ///
    /// The transform expression must have one input mapping and two output mappings.
    ///
    /// Example `transform_expr`: `tr!(seq1.* -> seq1.left, seq1.right)`.
    fn cut(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        cut_idx: EndIdx,
    ) -> CutReads<Self>

    /// Intersect two mapping intervals and create a new mapping of the intersection, if it is not empty.
    ///
    /// The transform expression must have two input mappings and one output mapping.
    ///
    /// Example `transform_expr`: `tr!(seq1.a, seq1.b -> seq1.c)`.
    fn intersect(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
    ) -> IntersectReads<Self>

    /// Union two mapping intervals and create a new mapping of the union.
    ///
    /// If the two mapping intervals are disjoint, then the union will also contain the region
    /// between the two mapping intervals, which is not inside either mapping intervals.
    ///
    /// The transform expression must have two input mappings and one output mapping.
    ///
    /// Example `transform_expr`: `tr!(seq1.a, seq1.b -> seq1.c)`.
    fn union(self, selector_expr: SelectorExpr, transform_expr: TransformExpr) -> UnionReads<Self>

    /// Trim the mappings corresponding to the specified labels by modifying the underlying strings.
    ///
    /// When a mapping is trimmed, its length will be set to zero. All intersecting
    /// mappings will also be adjusted accordingly for the shortening.
    fn trim(self, selector_expr: SelectorExpr, labels: impl Into<Vec<Label>>) -> TrimReads<Self>

    /// Set a label or attribute to the result of a format expression.
    ///
    /// After a label is set, its mapping and all other intersecting mappings will be adjusted accordingly
    /// for any shortening or lengthening.
    fn set(
        self,
        selector_expr: SelectorExpr,
        label_or_attr: impl Into<LabelOrAttr>,
        format_expr: impl AsRef<str>,
    ) -> SetReads<Self>

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
    fn match_regex(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        regex: impl AsRef<str>,
    ) -> MatchRegexReads<Self>

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
    fn match_any(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        patterns_yaml: impl AsRef<str>,
        match_type: MatchType,
    ) -> MatchAnyReads<Self>

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
    fn match_one(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        pattern: impl AsRef<str>,
        match_type: MatchType,
    ) -> MatchAnyReads<Self>

    /// Match repeated characters from the left or right end of a mapping.
    ///
    /// The transform expression must have one input mapping and two output mappings.
    ///
    /// Example `transform_expr`: `tr!(seq1.* -> seq1.sequence, seq1.polya_tail)`.
    fn match_polyx(
        self,
        selector_expr: SelectorExpr,
        transform_expr: TransformExpr,
        x: char,
        end: End,
        identity: f64,
    ) -> MatchPolyXReads<Self>

    /// Output reads to a specified file.
    ///
    /// The file path is a format expression.
    ///
    /// Only read 1 is written out.
    fn collect_fastq1(
        self,
        selector_expr: SelectorExpr,
        file_expr: impl AsRef<str>,
    ) -> CollectFastqReads<Self>

    /// Output paired-end reads to the specified files.
    ///
    /// The file paths are format expressions.
    ///
    /// Read 1 is written to `file_expr1` and read 2 is written to `file_expr2`.
    /// The reads will be interleaved if the files are the same.
    fn collect_fastq2(
        self,
        selector_expr: SelectorExpr,
        file_expr1: impl AsRef<str>,
        file_expr2: impl AsRef<str>,
    ) -> CollectFastqReads<Self>

    /// Retain only the reads that are selected and discard the rest.
    fn retain(self, selector_expr: SelectorExpr) -> RetainReads<Self>

    /// Take only the reads that have a record index inside the bounds.
    fn take<B>(self, bounds: B) -> TakeReads<Self, B>

    /// Create two read iterators by cloning each read.
    ///
    /// You must use the [`run!()`](crate::run!) or [`run_with_threads!()`](crate::run_with_threads!) macros to run all the forks.
    fn fork(self) -> (ForkReads<Self>, ForkReads<Self>)

    /// Compute the runtime (in seconds) of all operations before this in the iterator chain.
    ///
    /// The runtime is summed across all threads.
    ///
    /// The function `func` is called at the end with the runtime.
    fn time<F>(self, func: F) -> TimeReads<Self, F>
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
