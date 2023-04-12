use needletail::*;

use std::sync::Mutex;

use crate::{Read, Reads};

pub struct FastqReads {
    reader: Mutex<Box<dyn FastxReader>>,
    chunk_size: usize,
}

impl Reads for FastqReads {
    fn next_chunk(&self) -> Vec<Read> {
        let mut res = Vec::with_capacity(self.chunk_size);

        let mut reader = self.reader.lock().unwrap();
        for _ in 0..self.chunk_size {
            if let Some(record) = reader.next() {
                let record = record.unwrap();
                res.push(Read::from_fastq(record.id(), record.seq(), record.qual().unwrap()));
            } else {
                break;
            }
        }

        res
    }
}

#[must_use]
pub fn iter_fastq(file: &str, chunk_size: usize) -> FastqReads {
    let reader = Mutex::new(parse_fastx_file(file).expect("Error parsing input file!"));
    FastqReads {
        reader,
        chunk_size,
    }
}
