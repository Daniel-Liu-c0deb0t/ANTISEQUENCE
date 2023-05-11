//! Rust library for preprocessing sequencing reads.

pub mod expr;
pub mod fastq;
pub mod iter;
pub mod patterns;
pub mod read;

mod inline_string;

/// Easily import all commonly used types and functions.
pub mod prelude {
    pub use crate::expr::*;
    pub use crate::fastq::*;
    pub use crate::iter::*;
    pub use crate::patterns::*;
    pub use crate::read::*;
}
