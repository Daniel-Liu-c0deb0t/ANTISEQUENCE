use antisequence::*;

fn main() {
    let pattern_1 = r#"
        name: adapter
        patterns:
            - pattern: "CAGAGC"
    "#;

    let pattern_2 = r#"
        name: adapter_2
        patterns:
            - pattern: "GACTC"
    "#;

    iter_fastq1("example_data/pad.fastq", 256)
        .unwrap_or_else(|e| panic!("{e}"))
        .match_any(
            sel!(), 
            tr!(seq1.* -> seq1.rest, seq1.adapter_2), 
            pattern_2, 
            SuffixAln { identity: 0.83, overlap: 1.0 }
        )
        .match_any(
            sel!(),
            tr!(seq1.rest -> seq1.template, seq1.adapter, seq1.rest2),
            pattern_1,
            LocalAln {
                identity: 0.75,
                overlap: 1.0,
            },
        )
        .pad(sel!(), [label!(seq1.template)], 12)
        .dbg(sel!())
        .collect_fastq1(sel!(), "example_output/pad.fastq")
        .run()
        .unwrap_or_else(|e| panic!("{e}"));
}
