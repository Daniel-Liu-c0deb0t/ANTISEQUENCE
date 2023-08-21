use needletail::*;

use thread_local::*;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::collections::VecDeque;

use crate::errors::*;
use crate::graph::*;
use crate::expr::LabelOrAttr;

const CHUNK_SIZE: usize = 256;

pub struct InputFastq1Node<'reader> {
    reader: Mutex<Box<dyn FastxReader + 'reader>>,
    buf: ThreadLocal<RefCell<VecDeque<Read>>>,
    origin: Arc<Origin>,
    idx: AtomicUsize,
    interleaved: bool,
}

impl<'reader> InputFastq1Node<'reader> {
    const NAME: &'static str = "InputFastq1Node";

    /// Stream reads created from fastq records from an input file.
    pub fn new(file: impl AsRef<str>) -> Result<Self> {
        let reader = Mutex::new(parse_fastx_file(file.as_ref()).map_err(|e| Error::FileIo {
            file: file.as_ref().to_owned(),
            source: Box::new(e),
        })?);

        Ok(Self {
            reader,
            buf: ThreadLocal::new(),
            origin: Arc::new(Origin::File(file.as_ref().to_owned())),
            idx: AtomicUsize::new(0),
            interleaved: false,
        })
    }

    /// Stream reads created from interleaved paired-end fastq records from an input file.
    pub fn new_interleaved(
        file: impl AsRef<str>,
    ) -> Result<Self> {
        let reader = Mutex::new(parse_fastx_file(file.as_ref()).map_err(|e| Error::FileIo {
            file: file.as_ref().to_owned(),
            source: Box::new(e),
        })?);

        Ok(Self {
            reader,
            buf: ThreadLocal::new(),
            origin: Arc::new(Origin::File(file.as_ref().to_owned())),
            idx: AtomicUsize::new(0),
            interleaved: true,
        })
    }

    /// Stream reads created from fastq records from a byte slice.
    pub fn from_bytes(bytes: &'reader [u8]) -> Result<Self> {
        let reader = Mutex::new(parse_fastx_reader(bytes).map_err(|e| Error::BytesIo(Box::new(e)))?);

        Ok(Self {
            reader,
            buf: ThreadLocal::new(),
            origin: Arc::new(Origin::Bytes),
            idx: AtomicUsize::new(0),
            interleaved: false,
        })
    }

    /// Stream reads created from interleaved paired-end fastq records from a byte slice.
    pub fn from_interleaved_bytes(bytes: &'reader [u8]) -> Result<Self> {
        let reader = Mutex::new(parse_fastx_reader(bytes).map_err(|e| Error::BytesIo(Box::new(e)))?);

        Ok(Self {
            reader,
            buf: ThreadLocal::new(),
            origin: Arc::new(Origin::Bytes),
            idx: AtomicUsize::new(0),
            interleaved: true,
        })
    }
}

impl<'reader> GraphNode for InputFastq1Node<'reader> {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        assert!(read.is_none(), "Expected no input reads for {}", Self::NAME);

        let buf = self.buf.get_or(|| RefCell::new(VecDeque::with_capacity(CHUNK_SIZE)));
        let mut b = buf.borrow_mut();

        if b.is_empty() {
            let mut record1_id = Vec::new();
            let mut record1_seq = Vec::new();
            let mut record1_qual = Vec::new();
            let mut reader = self.reader.lock().unwrap();

            for _ in 0..CHUNK_SIZE {
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

                        b.push_back(Read::from_fastq1(
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

                    b.push_back(Read::from_fastq2(
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
        }

        if b.is_empty() {
            return Ok((None, true));
        }

        Ok((b.pop_front(), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}

pub struct InputFastq2Node {
    reader1: Mutex<Box<dyn FastxReader>>,
    reader2: Mutex<Box<dyn FastxReader>>,
    buf: ThreadLocal<RefCell<VecDeque<Read>>>,
    origin1: Arc<Origin>,
    origin2: Arc<Origin>,
    idx: AtomicUsize,
}

impl InputFastq2Node {
    const NAME: &'static str = "InputFastq2Node";

    /// Stream reads created from paired-end fastq records from two different input files.
    pub fn new(
        file1: impl AsRef<str>,
        file2: impl AsRef<str>,
    ) -> Result<Self> {
        let reader1 = Mutex::new(parse_fastx_file(file1.as_ref()).map_err(|e| Error::FileIo {
            file: file1.as_ref().to_owned(),
            source: Box::new(e),
        })?);
        let reader2 = Mutex::new(parse_fastx_file(file2.as_ref()).map_err(|e| Error::FileIo {
            file: file2.as_ref().to_owned(),
            source: Box::new(e),
        })?);

        Ok(Self {
            reader1,
            reader2,
            buf: ThreadLocal::new(),
            origin1: Arc::new(Origin::File(file1.as_ref().to_owned())),
            origin2: Arc::new(Origin::File(file2.as_ref().to_owned())),
            idx: AtomicUsize::new(0),
        })
    }
}

impl GraphNode for InputFastq2Node {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        assert!(read.is_none(), "Expected no input reads for {}", Self::NAME);

        let buf = self.buf.get_or(|| RefCell::new(VecDeque::with_capacity(CHUNK_SIZE)));
        let mut b = buf.borrow_mut();

        if b.is_empty() {
            let mut reader1 = self.reader1.lock().unwrap();
            let mut reader2 = self.reader2.lock().unwrap();

            for _ in 0..CHUNK_SIZE {
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

                b.push_back(Read::from_fastq2(
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
        }

        if b.is_empty() {
            return Ok((None, true));
        }

        Ok((b.pop_front(), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
