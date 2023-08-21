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
    /// A match will result in one new interval: the entire string.
    Exact,
    /// Exact prefix match.
    ///
    /// A match will result in two new interval: the matched prefix and the rest of the string.
    ExactPrefix,
    /// Exact suffix match.
    ///
    /// A match will result in two new interval: the rest of the string and the matched
    /// suffix.
    ExactSuffix,
    /// Exact match search.
    ///
    /// A match will result in three new interval: everything before the exact match, the exact
    /// matching region, everything after the exact match.
    ExactSearch,
    /// Hamming-distance-based matching.
    ///
    /// Threshold is for the number of matching bases.
    ///
    /// A match will result in one new interval: the entire string.
    Hamming(Threshold),
    /// Hamming-distance-based prefix matching.
    ///
    /// Threshold is for the number of matching bases.
    ///
    /// A match will result in two new interval: the matched prefix and the rest of the
    /// string.
    HammingPrefix(Threshold),
    /// Hamming-distance-based suffix matching.
    ///
    /// Threshold is for the number of matching bases.
    ///
    /// A match will result in two new interval: the rest of the string and the matched
    /// suffix.
    HammingSuffix(Threshold),
    /// Hamming-distance-based searching.
    ///
    /// Threshold is for the number of matching bases.
    ///
    /// A match will result in three new interval: everything before the match, the matching
    /// region, and everything after the match.
    HammingSearch(Threshold),
    /// Global-alignment-based matching.
    ///
    /// Threshold is for the sequence identity.
    ///
    /// A match will result in one new interval: the entire string.
    GlobalAln(f64),
    /// Local-alignment-based matching.
    ///
    /// A match will result in three new interval: everything before the aligned region, the locally aligned
    /// region, and everything after the aligned region.
    LocalAln { identity: f64, overlap: f64 },
    /// Prefix-alignment-based matching.
    ///
    /// A match will result in two new interval: the matched prefix and the rest of the
    /// string.
    PrefixAln { identity: f64, overlap: f64 },
    /// Suffix-alignment-based matching.
    ///
    /// A match will result in two new interval: the rest of the string and the matched
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
