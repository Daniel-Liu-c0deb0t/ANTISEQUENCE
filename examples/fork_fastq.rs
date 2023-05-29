use antisequence::*;

fn main() {
    let (left, right) = iter_fastq1("example_data/simple.fastq", 1)
        .unwrap_or_else(|e| panic!("{e}"))
        .cut(sel!(), tr!(seq1.* -> seq1.a, seq1.b), LeftEnd(3))
        .fork();

    let left = left
        .set(sel!(), label!(name1.*), "{name1.*}_{seq1.a}")
        .dbg(sel!());

    let right = right.trim(sel!(), [label!(seq1.a)]).dbg(sel!());

    run!(left, right);
}
