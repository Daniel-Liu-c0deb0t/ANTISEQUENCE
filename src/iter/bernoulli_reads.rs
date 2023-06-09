use rand::distributions::Bernoulli;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::iter::*;

pub struct BernoulliReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    attr: Attr,
    bernoulli: Bernoulli,
    seed: u64,
}

impl<R: Reads> BernoulliReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr, attr: Attr, prob: f64, seed: u32) -> Self {
        Self {
            reads,
            selector_expr,
            attr,
            bernoulli: Bernoulli::new(prob)
                .unwrap_or_else(|e| panic!("Error creating bernoulli distribution: {e}")),
            seed: seed as u64,
        }
    }
}

impl<R: Reads> Reads for BernoulliReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        // use the index of the read in the seed for determinism when multithreading
        let seed = (self.seed << 32)
            .wrapping_add(reads.first().map(|r| r.first_idx() as u64).unwrap_or(0u64));
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "generating bernoulli random samples",
                })?)
            {
                continue;
            }

            let rand_bool = self.bernoulli.sample(&mut rng);

            // panic to make borrow checker happy
            *read
                .data_mut(self.attr.str_type, self.attr.label, self.attr.attr)
                .unwrap_or_else(|e| panic!("Error generating bernoulli random samples: {e}")) =
                Data::Bool(rand_bool);
        }

        Ok(reads)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()
    }
}
