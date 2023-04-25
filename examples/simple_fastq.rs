use antisequence::{fastq::*, iter::*, read::*};

fn main() {
    iter_fastq("example_data/simple.fastq", 256)
        .cut("", "seq.* -> seq.a, seq.b", LeftEnd(3))
        .cut("", "seq.b -> _, seq.c", RightEnd(4))
        .for_each("", |read| println!("{}", read))
        .set("", "name.*", "{name.*}_{seq.a}")
        .trim("", ["seq.a"])
        .for_each("", |read| println!("{}", read))
        .collect_fastq("", "example_output/simple.fastq")
        .run(1);
}
