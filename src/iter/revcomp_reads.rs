use lazy_static::lazy_static;

use crate::iter::*;

lazy_static! {
    pub static ref COMPLEMENT: [u8; 256] = {
        let mut comp = [0; 256];

        for (v, a) in comp.iter_mut().enumerate() {
            *a = v as u8;
        }

        // IUPAC DNA alphabet
        for (&a, &b) in b"AGCTYRWSKMDVHBN".iter().zip(b"TCGARYWSMKHBDVN".iter()) {
            comp[a as usize] = b; // upper case
            comp[a as usize + 32] = b + 32; // lower case
        }

        comp
    };
}

pub struct RevCompReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    labels: Vec<Label>,
}

impl<R: Reads> RevCompReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr, labels: Vec<Label>) -> Self {
        Self {
            reads,
            selector_expr,
            labels,
        }
    }
}

impl<R: Reads> Reads for RevCompReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut reads = self.reads.next_chunk()?;

        for read in reads.iter_mut() {
            if !(self
                .selector_expr
                .matches(read)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "revcomp reads",
                })?)
            {
                continue;
            }

            self.labels
                .iter()
                .try_for_each(|l| read.revcomp(l.str_type, l.label))
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "revcomp reads",
                })?;
        }

        Ok(reads)
    }

    fn finish(&mut self) -> Result<()> {
        self.reads.finish()
    }
}
