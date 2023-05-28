use crate::iter::*;

pub struct TakeReads<R: Reads, B: RangeBounds<usize> + std::marker::Sync> {
    reads: R,
    bounds: B,
}

impl<R: Reads, B: RangeBounds<usize> + std::marker::Sync> TakeReads<R, B> {
    pub fn new(reads: R, bounds: B) -> Self {
        Self { reads, bounds }
    }
}

impl<R: Reads, B: RangeBounds<usize> + std::marker::Sync> Reads for TakeReads<R, B> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;
        reads.retain(|r| self.bounds.contains(&r.first_idx()));
        Ok(reads)
    }

    fn finish(&self) -> Result<()> {
        self.reads.finish()
    }
}
