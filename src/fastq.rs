use needletail::*;

use std::fmt;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::errors::*;
use crate::iter::*;
use crate::read::*;

pub struct Fastq1Reads<'a> {
    reader: Mutex<Box<dyn FastxReader + 'a>>,
    origin: Arc<Origin>,
    idx: AtomicUsize,
    chunk_size: usize,
    interleaved: bool,
}

impl<'a> Reads for Fastq1Reads<'a> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut res = Vec::with_capacity(self.chunk_size);
        let mut record1_id = Vec::new();
        let mut record1_seq = Vec::new();
        let mut record1_qual = Vec::new();

        let mut reader = self.reader.lock().unwrap();

        for _ in 0..self.chunk_size {
            if let Some(record1) = reader.next() {
                let record1 = record1.map_err(|e| Error::ParseRecord {
                    origin: (*self.origin).clone(),
                    idx: self.idx.load(Ordering::Relaxed),
                    source: Box::new(e),
                })?;

                if self.interleaved {
                    record1_id.clear();
                    record1_id.extend_from_slice(record1.id());
                    record1_seq.clear();
                    record1_seq.extend_from_slice(&record1.seq());
                    record1_qual.clear();
                    record1_qual.extend_from_slice(record1.qual().unwrap());
                } else {
                    let idx = self.idx.fetch_add(1, Ordering::Relaxed);

                    res.push(Read::from_fastq1(
                        record1.id(),
                        &record1.seq(),
                        record1.qual().unwrap(),
                        Arc::clone(&self.origin),
                        idx,
                    ));
                }
            } else {
                break;
            }

            if self.interleaved {
                let Some(record2) = reader.next() else {
                    Err(Error::UnpairedRead(format!("\"{}\"", &*self.origin)))?
                };
                let record2 = record2.map_err(|e| Error::ParseRecord {
                    origin: (*self.origin).clone(),
                    idx: self.idx.load(Ordering::Relaxed) + 1,
                    source: Box::new(e),
                })?;
                let idx = self.idx.fetch_add(2, Ordering::Relaxed);

                res.push(Read::from_fastq2(
                    &record1_id,
                    &record1_seq,
                    &record1_qual,
                    Arc::clone(&self.origin),
                    idx,
                    record2.id(),
                    &record2.seq(),
                    record2.qual().unwrap(),
                    Arc::clone(&self.origin),
                    idx + 1,
                ));
            }
        }

        Ok(res)
    }

    fn finish(self) -> Result<()> {
        Ok(())
    }
}

pub struct Fastq2Reads {
    reader1: Mutex<Box<dyn FastxReader>>,
    reader2: Mutex<Box<dyn FastxReader>>,
    origin1: Arc<Origin>,
    origin2: Arc<Origin>,
    idx: AtomicUsize,
    chunk_size: usize,
}

impl Reads for Fastq2Reads {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let mut res = Vec::with_capacity(self.chunk_size);

        let mut reader1 = self.reader1.lock().unwrap();
        let mut reader2 = self.reader2.lock().unwrap();

        for _ in 0..self.chunk_size {
            let Some(record1) = reader1.next() else {
                break;
            };
            let Some(record2) = reader2.next() else {
                Err(Error::UnpairedRead(format!("\"{}\" and \"{}\"", &*self.origin1, &*self.origin2)))?
            };

            let record1 = record1.map_err(|e| Error::ParseRecord {
                origin: (*self.origin1).clone(),
                idx: self.idx.load(Ordering::Relaxed),
                source: Box::new(e),
            })?;
            let record2 = record2.map_err(|e| Error::ParseRecord {
                origin: (*self.origin2).clone(),
                idx: self.idx.load(Ordering::Relaxed),
                source: Box::new(e),
            })?;
            let idx = self.idx.fetch_add(1, Ordering::Relaxed);

            res.push(Read::from_fastq2(
                record1.id(),
                &record1.seq(),
                record1.qual().unwrap(),
                Arc::clone(&self.origin1),
                idx,
                record2.id(),
                &record2.seq(),
                record2.qual().unwrap(),
                Arc::clone(&self.origin2),
                idx,
            ));
        }

        Ok(res)
    }

    fn finish(self) -> Result<()> {
        Ok(())
    }
}

#[must_use]
pub fn iter_fastq1(file: impl AsRef<str>, chunk_size: usize) -> Result<Fastq1Reads<'static>> {
    let reader = Mutex::new(parse_fastx_file(file.as_ref()).map_err(|e| Error::FileIo {
        file: file.as_ref().to_owned(),
        source: Box::new(e),
    })?);
    Ok(Fastq1Reads::<'static> {
        reader,
        origin: Arc::new(Origin::File(file.as_ref().to_owned())),
        idx: AtomicUsize::new(0),
        chunk_size,
        interleaved: false,
    })
}

#[must_use]
pub fn iter_fastq_interleaved(
    file: impl AsRef<str>,
    chunk_size: usize,
) -> Result<Fastq1Reads<'static>> {
    let reader = Mutex::new(parse_fastx_file(file.as_ref()).map_err(|e| Error::FileIo {
        file: file.as_ref().to_owned(),
        source: Box::new(e),
    })?);
    Ok(Fastq1Reads::<'static> {
        reader,
        origin: Arc::new(Origin::File(file.as_ref().to_owned())),
        idx: AtomicUsize::new(0),
        chunk_size,
        interleaved: true,
    })
}

#[must_use]
pub fn iter_fastq2(
    file1: impl AsRef<str>,
    file2: impl AsRef<str>,
    chunk_size: usize,
) -> Result<Fastq2Reads> {
    let reader1 = Mutex::new(parse_fastx_file(file1.as_ref()).map_err(|e| Error::FileIo {
        file: file1.as_ref().to_owned(),
        source: Box::new(e),
    })?);
    let reader2 = Mutex::new(parse_fastx_file(file2.as_ref()).map_err(|e| Error::FileIo {
        file: file2.as_ref().to_owned(),
        source: Box::new(e),
    })?);
    Ok(Fastq2Reads {
        reader1,
        reader2,
        origin1: Arc::new(Origin::File(file1.as_ref().to_owned())),
        origin2: Arc::new(Origin::File(file2.as_ref().to_owned())),
        idx: AtomicUsize::new(0),
        chunk_size,
    })
}

#[must_use]
pub fn iter_fastq1_bytes<'a>(bytes: &'a [u8]) -> Result<Fastq1Reads<'a>> {
    let reader = Mutex::new(parse_fastx_reader(bytes).map_err(|e| Error::BytesIo(Box::new(e)))?);
    Ok(Fastq1Reads::<'a> {
        reader,
        origin: Arc::new(Origin::Bytes),
        idx: AtomicUsize::new(0),
        chunk_size: 256,
        interleaved: false,
    })
}

#[must_use]
pub fn iter_fastq_interleaved_bytes<'a>(bytes: &'a [u8]) -> Result<Fastq1Reads<'a>> {
    let reader = Mutex::new(parse_fastx_reader(bytes).map_err(|e| Error::BytesIo(Box::new(e)))?);
    Ok(Fastq1Reads::<'a> {
        reader,
        origin: Arc::new(Origin::Bytes),
        idx: AtomicUsize::new(0),
        chunk_size: 256,
        interleaved: true,
    })
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

#[derive(Debug, Clone)]
pub enum Origin {
    File(String),
    Bytes,
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Origin::File(file) => write!(f, "file: \"{}\"", file),
            Origin::Bytes => write!(f, "bytes"),
        }
    }
}
