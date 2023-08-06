use rand::distributions::Bernoulli;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::iter::*;

pub struct BernoulliNode {
    attr: Attr,
    bernoulli: Bernoulli,
    seed: u64,
}

impl BernoulliNode {
    const NAME: 'static str = "generating bernoulli random samples";

    pub fn new(attr: Attr, prob: f64, seed: u32) -> Self {
        Self {
            attr,
            bernoulli: Bernoulli::new(prob)
                .unwrap_or_else(|e| panic!("Error creating bernoulli distribution: {e}")),
            seed: seed as u64,
        }
    }
}

impl GraphNode for BernoulliNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(mut read) = read else { panic!("Expected some read!") };

        // use the index of the read in the seed for determinism when multithreading
        let seed = (self.seed << 32)
            .wrapping_add(reads.first().map(|r| r.first_idx() as u64).unwrap_or(0u64));
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        let rand_bool = self.bernoulli.sample(&mut rng);

        // panic to make borrow checker happy
        *read
            .data_mut(self.attr.str_type, self.attr.label, self.attr.attr)
            .unwrap_or_else(|e| panic!("Error when {}: {e}", Self::NAME)) =
            Data::Bool(rand_bool);

        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
