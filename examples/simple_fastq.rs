use antisequence::prelude::*;

fn main() {
    iter_fastq1("example_data/simple.fastq", 256)
        .cut("", "seq1.* -> seq1.a, seq1.b", LeftEnd(3))
        .cut("", "seq1.b -> _, seq1.c", RightEnd(4))
        .for_each("", |read| println!("{}", read))
        .set("", "name1.*", "{name1.*}_{seq1.a}")
        .trim("", ["seq1.a"])
        .for_each("", |read| println!("{}", read))
        .collect_fastq1("", "example_output/simple.fastq")
        .run(1);
}
