use std::thread;

use crate::expr::*;
use crate::read::*;

pub mod trim_reads;
pub use trim_reads::*;

pub mod collect_fastq_reads;
pub use collect_fastq_reads::*;

pub trait Reads: Sized + std::marker::Sync {
    fn run(self, threads: usize) {
        assert!(threads >= 1);

        thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| while self.next_chunk().len() > 0 {});
            }
        });
    }

    #[must_use]
    fn trim(&self, selector_expr: &str, labels: &[&str]) -> TrimReads<Self> {
        let labels = labels.iter().map(|l| Label::new(l)).collect::<Vec<_>>();
        TrimReads::new(self, SelectorExpr::new(selector_expr), labels)
    }

    #[must_use]
    fn collect_fastq(&self, selector_expr: &str, file_expr: &str) -> CollectFastqReads<Self> {
        CollectFastqReads::new(
            self,
            SelectorExpr::new(selector_expr),
            FormatExpr::new(file_expr),
        )
    }

    fn next_chunk(&self) -> Vec<Read>;
}
