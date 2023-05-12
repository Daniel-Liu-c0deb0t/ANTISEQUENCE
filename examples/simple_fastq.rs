use antisequence::*;

fn main() {
    iter_fastq1("example_data/simple.fastq", 256)
        .cut(sel!(), tr!(seq1.* -> seq1.a, seq1.b), LeftEnd(3))
        .cut(sel!(), tr!(seq1.b -> _, seq1.c), RightEnd(4))
        .dbg(sel!())
        .set(sel!(), label!(name1.*), "{name1.*}_{seq1.a}")
        .trim(sel!(), [label!(seq1.a)])
        .dbg(sel!())
        .collect_fastq1(sel!(), "example_output/simple.fastq")
        .run(1);
}
