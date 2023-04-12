//! Rust library for preprocessing sequencing reads.

use std::thread;

pub mod fastq;
pub mod preprocess;
pub mod read;

pub trait Reads {
    fn run(&self, threads: usize) {
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
    fn trim(&self, labels: &[&str]);

    #[must_use]
    fn collect_fastq(&self, label: &str, file: &str);

    fn next_chunk(&self) -> Vec<Read>;
}
