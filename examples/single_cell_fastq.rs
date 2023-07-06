use antisequence::*;

fn main() {
    // Demo single-cell sequencing protocol:
    // R1: bc[9-11] CAGAGC umi[8] bc[10]
    // R2: insert adapter

    let fastq = b"@read1/1
AAAAAAAAAACAGAGCTTTTTTTTCCCCCCCCCC
+
0123456789012345678901234567890123
@read1/2
AAAATTTTCCCCGGGGCGCGACG
+
01234567890123456789012
@read2/1
AAAAAAAAAAAAAACAGAGCTTTTTTTTCCCCCCCCCC
+
01234567890123456789012345678901234567
@read2/2
AAAATTTTCCCCGGGGATATAT
+
0123456789012345678901";

    let adapters = "
        name: adapter
        patterns:
          - pattern: ATATATATAT
          - pattern: CGCGCGCGCG
    ";

    iter_fastq_interleaved_bytes(fastq)
        .unwrap_or_else(|e| panic!("{e}"))
        // trim adapter
        .match_any(
            sel!(),
            tr!(seq2.* -> _, seq2.adapter),
            adapters,
            SuffixAln {
                identity: 0.9,
                overlap: 0.4,
            },
        )
        .dbg(sel!())
        .trim(sel!(seq2.adapter), [label!(seq2.adapter)])
        // match anchor
        .match_one(
            sel!(),
            tr!(seq1.* -> seq1.bc1, _, seq1.after_anchor),
            "CAGAGC",
            HammingSearch(Frac(0.8)),
        )
        // check the length of the first barcode
        .length_in_bounds(sel!(seq1.bc1), tr!(seq1.bc1 -> seq1.bc1.in_bounds), 9..=11)
        // split the UMI from the rest of the sequence
        .cut(
            sel!(seq1.after_anchor),
            tr!(seq1.after_anchor -> seq1.umi, seq1.after_umi),
            LeftEnd(8),
        )
        // clip the length of the second barcode
        .cut(
            sel!(seq1.after_umi),
            tr!(seq1.after_umi -> seq1.bc2, _),
            LeftEnd(10),
        )
        // check the length of the second barcode
        .length_in_bounds(sel!(seq1.bc2), tr!(seq1.bc2 -> seq1.bc2.in_bounds), 10..=10)
        .dbg(sel!())
        // filter out invalid reads
        .retain(sel!(
            seq1.bc1 & seq1.bc1.in_bounds & seq1.bc2 & seq1.bc2.in_bounds
        ))
        // move the UMI and barcodes to the read name
        .set(
            sel!(),
            label!(name1.*),
            "{name1.*}_{seq1.umi}_{seq1.bc1}{seq1.bc2}",
        )
        .set(sel!(), label!(seq1.*), "{seq2.*}")
        .collect_fastq1(sel!(), "example_output/single_cell.fastq")
        .run()
        .unwrap_or_else(|e| panic!("{e}"));
}
