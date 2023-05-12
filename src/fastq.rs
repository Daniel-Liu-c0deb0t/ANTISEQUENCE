use needletail::*;

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::iter::*;
use crate::read::*;

pub struct Fastq1Reads {
    reader: Mutex<Box<dyn FastxReader>>,
    file: Arc<String>,
    line: AtomicUsize,
    chunk_size: usize,
}

impl Reads for Fastq1Reads {
    fn next_chunk(&self) -> Vec<Read> {
        let mut res = Vec::with_capacity(self.chunk_size);

        let mut reader = self.reader.lock().unwrap();

        for _ in 0..self.chunk_size {
            if let Some(record) = reader.next() {
                let record = record.unwrap();
                let line = self.line.fetch_add(4, Ordering::SeqCst);

                res.push(Read::from_fastq1(
                    record.id(),
                    &record.seq(),
                    record.qual().unwrap(),
                    Arc::clone(&self.file),
                    line,
                ));
            } else {
                break;
            }
        }

        res
    }
}

pub struct Fastq2Reads {
    reader1: Mutex<Box<dyn FastxReader>>,
    reader2: Mutex<Box<dyn FastxReader>>,
    file1: Arc<String>,
    file2: Arc<String>,
    line: AtomicUsize,
    chunk_size: usize,
}

impl Reads for Fastq2Reads {
    fn next_chunk(&self) -> Vec<Read> {
        let mut res = Vec::with_capacity(self.chunk_size);

        let mut reader1 = self.reader1.lock().unwrap();
        let mut reader2 = self.reader2.lock().unwrap();

        for _ in 0..self.chunk_size {
            let Some(record1) = reader1.next() else {
                break;
            };
            let Some(record2) = reader2.next() else {
                panic!("Fastq files do not have the same number of lines!");
            };

            let record1 = record1.unwrap();
            let record2 = record2.unwrap();
            let line = self.line.fetch_add(4, Ordering::SeqCst);

            res.push(Read::from_fastq2(
                record1.id(),
                &record1.seq(),
                record1.qual().unwrap(),
                Arc::clone(&self.file1),
                line,
                record2.id(),
                &record2.seq(),
                record2.qual().unwrap(),
                Arc::clone(&self.file2),
                line,
            ));
        }

        res
    }
}

#[must_use]
pub fn iter_fastq1(file: impl AsRef<str>, chunk_size: usize) -> Fastq1Reads {
    let reader = Mutex::new(parse_fastx_file(file.as_ref()).expect("Error parsing input file!"));
    Fastq1Reads {
        reader,
        file: Arc::new(file.as_ref().to_owned()),
        line: AtomicUsize::new(0),
        chunk_size,
    }
}

#[must_use]
pub fn iter_fastq2(
    file1: impl AsRef<str>,
    file2: impl AsRef<str>,
    chunk_size: usize,
) -> Fastq2Reads {
    let reader1 = Mutex::new(parse_fastx_file(file1.as_ref()).expect("Error parsing input file1!"));
    let reader2 = Mutex::new(parse_fastx_file(file2.as_ref()).expect("Error parsing input file2!"));
    Fastq2Reads {
        reader1,
        reader2,
        file1: Arc::new(file1.as_ref().to_owned()),
        file2: Arc::new(file2.as_ref().to_owned()),
        line: AtomicUsize::new(0),
        chunk_size,
    }
}

pub fn write_fastq_record(
    writer: &mut (dyn Write + std::marker::Send),
    record: (&[u8], &[u8], &[u8]),
) {
    writer.write_all(b"@").unwrap();
    writer.write_all(&record.0).unwrap();
    writer.write_all(b"\n").unwrap();
    writer.write_all(&record.1).unwrap();
    writer.write_all(b"\n+\n").unwrap();
    writer.write_all(&record.2).unwrap();
    writer.write_all(b"\n").unwrap();
}
