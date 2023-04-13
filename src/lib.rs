//! Rust library for preprocessing sequencing reads.

use std::thread;

pub mod fastq;
pub mod preprocess;
pub mod read;

pub trait Reads {
    fn run(&self, threads: usize) {
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
    fn trim_seq(&self, labels: &[&str]) -> TrimReads {
        let labels = labels.iter().cloned().collect::<Vec<_>>();
        TrimReads::new(labels, true)
    }

    #[must_use]
    fn trim_name(&self, labels: &[&str]) -> TrimReads {
        let labels = labels.iter().cloned().collect::<Vec<_>>();
        TrimReads::new(labels, false)
    }

    #[must_use]
    fn collect_fastq(&self, selector_expr: &str, file: &str) -> CollectFastqReads {
        CollectFastqReads::new(SelectorExpr::new(label), file.to_owned())
    }

    fn next_chunk(&self) -> Vec<Read>;
}
