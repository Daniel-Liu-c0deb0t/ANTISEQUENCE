use std::sync::atomic::{AtomicUsize, Ordering};

use crate::iter::*;

pub struct CountReads<R: Reads, F: Fn(&[usize]) + std::marker::Sync> {
    reads: R,
    selector_exprs: Vec<SelectorExpr>,
    counts: Vec<AtomicUsize>,
    func: F,
}

impl<R: Reads, F: Fn(&[usize]) + std::marker::Sync> CountReads<R, F> {
    pub fn new(reads: R, selector_exprs: Vec<SelectorExpr>, func: F) -> Self {
        let counts = (0..selector_exprs.len())
            .map(|_| AtomicUsize::new(0))
            .collect();
        Self {
            reads,
            selector_exprs,
            counts,
            func,
        }
    }
}

impl<R: Reads, F: Fn(&[usize]) + std::marker::Sync> Reads for CountReads<R, F> {
    fn next_chunk(&self) -> Vec<Read> {
        let reads = self.reads.next_chunk();

        for read in &reads {
            for (c, s) in self.counts.iter().zip(&self.selector_exprs) {
                if s.matches(&read) {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        reads
    }

    fn finish(&self) -> Result<()> {
        self.reads.finish()?;

        let counts = self
            .counts
            .iter()
            .map(|c| c.load(Ordering::Relaxed))
            .collect::<Vec<_>>();
        (self.func)(&counts);
        Ok(())
    }
}
