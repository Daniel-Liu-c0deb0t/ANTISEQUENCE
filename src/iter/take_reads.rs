use crate::iter::*;

pub struct TakeReads<R: Reads> {
    reads: R,
    count: usize,
}

impl<R: Reads> TakeReads<R> {
    pub fn new(reads: R, count: usize) -> Self {
        Self { reads, count }
    }
}

impl<R: Reads> Reads for TakeReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        if let Some(read) = reads.first() {
            if read.first_idx() >= self.count {
                return Ok(Vec::new());
            }
        }

        if let Some(read) = reads.last() {
            if read.first_idx() >= self.count {
                reads.retain(|r| r.first_idx() < self.count);
            }
        }

        Ok(reads)
    }

    fn finish(&self) -> Result<()> {
        self.reads.finish()
    }
}
