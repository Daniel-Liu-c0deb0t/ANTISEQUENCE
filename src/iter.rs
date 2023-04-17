use std::thread;

pub mod trim_reads;
pub use trim_reads::*;

pub mod collect_fastq_reads;
pub use collect_fastq_reads::*;

pub trait Reads {
    fn run(self, threads: usize) {
        assert!(threads >= 1);
        let mut handles = Vec::with_capacity(threads);

        for _ in 0..threads {
            handles.push(thread::spawn(|| {
                while self.next_chunk().len() > 0 {}
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }

    #[must_use]
    fn trim(&self, selector_expr: &str, labels: &[&str]) -> TrimReads {
        let labels = labels.iter().cloned().collect::<Vec<_>>();
        TrimReads::new(self, SelectorExpr::new(selector_expr), labels)
    }

    #[must_use]
    fn collect_fastq(&self, selector_expr: &str, file_expr: &str) -> CollectFastqReads {
        CollectFastqReads::new(self, SelectorExpr::new(selector_expr), FormatExpr::new(file_expr))
    }

    fn next_chunk(&self) -> Vec<Read>;
}
