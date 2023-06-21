use antisequence::*;

fn main() {
    let patterns = r#"
        name: adapter
        patterns:
            - pattern: "CAGAGC"
    "#;

    iter_fastq1("example_data/pad.fastq", 256)
        .unwrap_or_else(|e| panic!("{e}"))
        .match_any(
            sel!(),
            tr!(seq1.* -> seq1.template, seq1.adapter, seq1.rest),
            patterns,
            LocalAln {
                identity: 0.75,
                overlap: 0.5,
            },
        )
        .pad(sel!(), [label!(seq1.template)], 12)
        .dbg(sel!())
        .collect_fastq1(sel!(), "example_output/pad.fastq")
        .run()
        .unwrap_or_else(|e| panic!("{e}"));
}
