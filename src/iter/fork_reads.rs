use thread_local::*;

use std::cell::RefCell;
use std::sync::Arc;

use crate::iter::*;

pub type ForkBuf = ThreadLocal<RefCell<(bool, Vec<Read>)>>;

pub struct ForkReads<R: Reads> {
    reads: Arc<R>,
    buf: Arc<ForkBuf>,
}

impl<R: Reads> ForkReads<R> {
    pub fn new(reads: Arc<R>, buf: Arc<ForkBuf>) -> Self {
        Self { reads, buf }
    }
}

impl<R: Reads> Reads for ForkReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let buf = self.buf.get_or(|| RefCell::new((false, Vec::new())));
        let mut b = buf.borrow_mut();

        if b.0 {
            b.0 = false;
            Ok(b.1.drain(..).collect())
        } else {
            let reads = self.reads.next_chunk()?;
            b.0 = true;
            b.1.extend(reads.iter().cloned());
            Ok(reads)
        }
    }

    fn finish(&mut self) -> Result<()> {
        if let Some(reads) = Arc::get_mut(&mut self.reads) {
            reads.finish()
        } else {
            Ok(())
        }
    }
}
