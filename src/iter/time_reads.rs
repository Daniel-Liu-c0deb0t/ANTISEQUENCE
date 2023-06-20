use thread_local::*;

use std::cell::Cell;
use std::time::{Duration, Instant};

use crate::iter::*;

pub struct TimeReads<R: Reads, F: Fn(f64) + Send + Sync> {
    reads: R,
    duration: ThreadLocal<Cell<Duration>>,
    func: F,
}

impl<R: Reads, F: Fn(f64) + Send + Sync> TimeReads<R, F> {
    pub fn new(reads: R, func: F) -> Self {
        Self {
            reads,
            duration: ThreadLocal::new(),
            func,
        }
    }
}

impl<R: Reads, F: Fn(f64) + Send + Sync> Reads for TimeReads<R, F> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let start = Instant::now();
        let reads = self.reads.next_chunk()?;
        let elapsed = start.elapsed();

        let duration = self.duration.get_or(|| Cell::new(Duration::default()));
        duration.set(duration.get() + elapsed);

        Ok(reads)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()?;

        let duration = self.duration.iter_mut().map(|c| c.get()).sum::<Duration>();
        (self.func)(duration.as_secs_f64());
        Ok(())
    }
}
