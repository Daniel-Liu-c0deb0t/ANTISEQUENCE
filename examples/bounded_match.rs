use antisequence::*;

fn main() {
    // 1{b[9-10]f[CAGAGC]r:}

    let pattern = r#"
        name: adapter
        patterns:
            - pattern: "CAGAGC"
    "#;

    iter_fastq1("./example_data/bounded_match.fastq", 256)
        .unwrap_or_else(|e| panic!("{e}"))
        .match_any(
            sel!(),
            tr!(seq1.* -> seq1._l, seq1.adapter, seq1._r),
            pattern,
            BoundedAln {
                identity: 1.0,
                overlap: 1.0,
                from: 9,
                to: 15,
            },
        )
        .dbg(sel!())
        .run()
        .unwrap_or_else(|e| panic!("{e}"));
}
