//! Rust library for preprocessing sequencing reads.
//!
//! # Overview
//! ANTISEQUENCE provides efficient and composable operations for manipulating fastq records.
//!
//! This is useful for:
//! * Processing reads for custom sequencing protocol development
//! * Unifying read formats from various sequencing protocols
//! * Writing fastq processing tools
//! * Debugging sequencing pipelines
//!
//! ## Iterator-like API
//! ANTISEQUENCE's main API is similar to Rust iterators, but it uses custom operations that
//! operate on reads and it supports easy multithreading.
//!
//! Typically, you would specify *operations* (read from fastq, trim reads, output to fastq, etc.)
//! on reads in a chain, and these are executed in order when you call
//! [`run()`](Reads::run) or [`run_with_threads()`](Reads::run_with_threads).
//!
//! See [`fastq`] for functions for reading fastq records and
//! see [`Reads`] for all the supported read iterator operations.
//!
//! ## Read structure
//! Here's an example fastq record:
//! ```
//! @read6
//! AATTCCGGAATTCCCAAAAG
//! +
//! 01234567890123456789
//! ```
//! The first, second, and fourth lines are the name, sequence, and quality scores, respectively.
//!
//! ANTISEQUENCE stores that record as an internal [`Read`] data structure:
//! ```
//! name1:
//!   *     |---|
//!   str:  read6
//!   from record 5 in file: "example_data/match.fastq"
//! seq1:
//!   *        |------------------|  adapter=AAAA
//!   template |-------------|
//!   adapter                 |---|
//!   str:     AATTCCGGAATTCCCAAAAG
//!   qual:    01234567890123456789
//!   from record 5 in file: "example_data/match.fastq"
//! ```
//!
//! Each `Read` is a set of *strings* of different *types*. Types help indicate whether the string is
//! a read sequence (`seq1`) or read name (`name1`).
//!
//! Each string has associated *mappings*. Each mapping is a *label* and an *interval* in the string.
//! For example, mappings can label the region where an adapter is found in the read sequence.
//! All strings start with a mapping labeled `*`, which spans the whole string.
//! You can refer to a mapping with `seq1.*`, `seq1.adapter`, `name1.*`, etc.
//!
//! A mapping can contain *attributes* that hold arbitrary metadata. This may include a boolean for
//! whether to filter the read, or the name of the pattern that the read matches.
//! You can refer to an attribute with `seq1.*.adapter`, etc.
//!
//! Note that for efficiency and simplicity, most ANTISEQUENCE operations only manipulate the mappings
//! and attributes. You can choose to modify the underlying strings afterwards.
//!
//! ## Selector expressions
//! Selector expressions allow you to filter for reads that satisfy a boolean expression.
//! This is useful for choosing which reads to output or trimming reads that have a certain
//! pattern.
//! Many operations take a selector expression as the first parameter, which specifies
//! the reads that the operation is performed on.
//!
//! Here are some example selector expressions:
//! * `sel!()`: select all reads
//! * `sel!(seq1.adapter)`: select only reads with the `adapter` mapping in its sequence
//! * `sel!(seq1.adapter & !seq1.*.discard)`: arbitrary boolean expression!
//!
//! ## Transform expressions
//! Transform expressions allow you to specify the names of the inputs and outputs
//! for an operation. For example, to cut a mapping interval and create two new mappings,
//! you can use `tr!(seq1.* -> seq1.left, seq1.right)`.
//!
//! ## Format expressions
//! Format expressions allow you to contruct new strings from mappings and attributes,
//! and they are similar to Rust's formatting syntax. For example, you can use `"{seq1.a}_{seq1.b}"`
//! to concatenate the substrings corresponding to mappings `a` and `b`, separated by an
//! underscore. A string can also be repeated, like `"{'A'; 4}"`, which results in `AAAA`.
//!
//! Format expressions are useful for rearranging and modifying strings.
//! They also preserve quality scores, making rearranging regions in a read easy.

pub mod errors;
pub mod expr;
pub mod fastq;
pub mod iter;
pub mod patterns;
pub mod read;

mod inline_string;
mod parse_utils;

// commonly used functions and types

pub use crate::fastq::*;
pub use crate::iter::*;
pub use crate::patterns::*;
pub use crate::read::*;
