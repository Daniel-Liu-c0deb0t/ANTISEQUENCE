use antisequence::*;

fn main() {
    iter_fastq1("example_data/normalize.fastq", 256)
        .unwrap_or_else(|e| panic!("{e}"))
        .norm(sel!(), label!(seq1.*), (6, 10))
        .dbg(sel!())
        .run()
        .unwrap_or_else(|e| panic!("{e}"));
}
