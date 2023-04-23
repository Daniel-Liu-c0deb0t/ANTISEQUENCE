use std::thread;

use crate::expr::*;
use crate::read::*;

pub mod trim_reads;
pub use trim_reads::*;

pub mod collect_fastq_reads;
pub use collect_fastq_reads::*;

pub mod for_each_reads;
pub use for_each_reads::*;

pub mod cut_reads;
pub use cut_reads::*;

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
    fn for_each<F>(self, selector_expr: &str, func: F) -> ForEachReads<Self, F>
    where
        F: Fn(&mut Read) + std::marker::Sync,
    {
        ForEachReads::new(self, SelectorExpr::new(selector_expr), func)
    }

    #[must_use]
    fn cut(self, selector_expr: &str, transform_expr: &str, cut_idx: EndIdx) -> CutReads<Self> {
        CutReads::new(
            self,
            SelectorExpr::new(selector_expr),
            TransformExpr::new(transform_expr),
            cut_idx,
        )
    }

    #[must_use]
    fn trim<S>(self, selector_expr: &str, labels: impl AsRef<[S]>) -> TrimReads<Self> where S: AsRef<str> {
        let labels = labels.as_ref().iter().map(|l| Label::new(l.as_ref())).collect::<Vec<_>>();
        TrimReads::new(self, SelectorExpr::new(selector_expr), labels)
    }

    #[must_use]
    fn collect_fastq(self, selector_expr: &str, file_expr: &str) -> CollectFastqReads<Self> {
        CollectFastqReads::new(
            self,
            SelectorExpr::new(selector_expr),
            FormatExpr::new(file_expr),
        )
    }

    fn next_chunk(&self) -> Vec<Read>;
}
