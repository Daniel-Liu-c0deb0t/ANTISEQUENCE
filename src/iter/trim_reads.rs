use crate::read::*;

pub struct TrimReads<'r, R: Reads> {
    reads: &'r R,
    selector_expr: SelectorExpr,
    labels: Vec<String>,
    seq: bool,
}

impl<'r, R: Reads> TrimReads<'r, R> {
    pub fn new(reads: &'r R, selector_expr: SelectorExpr, labels: String, seq: bool) -> Self {
        Self {
            reads,
            selector_expr,
            labels,
            seq,
        }
    }
}

impl<'r, R: Reads> Reads for TrimReads<'r, R> {
    fn next_chunk() -> Read {
        let mut reads = self.reads.next_chunk();

        for read in &mut reads {
            if self.seq {
                self.labels.iter().for_each(|l| read.trim_seq(l));
            } else {
                self.labels.iter().for_each(|l| read.trim_name(l));
            }
        }

        reads
    }
}
