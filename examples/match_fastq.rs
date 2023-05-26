use antisequence::*;

fn main() {
    let patterns = r#"
        name: adapter
        patterns:
            - pattern: "AAAA"
            - pattern: "TTTT"
    "#;

    iter_fastq1("example_data/match.fastq", 256)
        .unwrap_or_else(|e| panic!("{e}"))
        .match_any(
            sel!(),
            tr!(seq1.* -> seq1.template, seq1.adapter),
            patterns,
            SuffixAln {
                identity: 0.75,
                overlap: 0.5,
            },
        )
        .dbg(sel!())
        .trim(sel!(seq1.adapter), [label!(seq1.adapter)])
        .dbg(sel!())
        .collect_fastq1(sel!(), "example_output/match.fastq")
        .run()
        .unwrap_or_else(|e| panic!("{e}"));
}
