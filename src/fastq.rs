use needletail::*;

use thread_local::*;

use std::fmt;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::collections::VecDeque;

use crate::errors::*;
use crate::iter::*;
use crate::read::*;
use crate::expr::{LabelOrAttr, Node};

type ReadBuf = ThreadLocal<RefCell<VecDeque<Read>>>;
const CHUNK_SIZE: usize = 256;

pub struct Fastq1Node<'reader> {
    next_node: Option<Box<dyn GraphNode>>,

    reader: Mutex<Box<dyn FastxReader + 'reader>>,
    buf: ReadBuf,
    origin: Arc<Origin>,
    idx: AtomicUsize,
    interleaved: bool,
}

impl<'reader> GraphNode for Fastq1Node<'reader> {
    fn run<'a>(&'a self, read: Option<Read>, next_nodes: &mut Vec<&'a dyn GraphNode>) -> Result<(Option<Read>, bool)> {
        assert!(read.is_none(), "Expected no input reads when {}", self.name());

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

        if let Some(node) = &self.next_node {
            next_nodes.push(&**node);
        }
        Ok((b.pop_front(), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn cond(&self) -> Option<Node> {
        None
    }

    fn set_next(&mut self, node: Box<dyn GraphNode>) -> &mut dyn GraphNode {
        self.next_node = Some(node);
        &mut **self.next_node.as_mut().unwrap()
    }

    fn name(&self) -> &'static str {
        "iterate reads from one fastq file"
    }
}

pub struct Fastq2Node {
    next_node: Option<Box<dyn GraphNode>>,

    reader1: Mutex<Box<dyn FastxReader>>,
    reader2: Mutex<Box<dyn FastxReader>>,
    buf: ReadBuf,
    origin1: Arc<Origin>,
    origin2: Arc<Origin>,
    idx: AtomicUsize,
}

impl GraphNode for Fastq2Node {
    fn run<'a>(&'a self, read: Option<Read>, next_nodes: &mut Vec<&'a dyn GraphNode>) -> Result<(Option<Read>, bool)> {
        assert!(read.is_none(), "Expected no input reads when {}", self.name());

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

        if let Some(node) = &self.next_node {
            next_nodes.push(&**node);
        }
        Ok((b.pop_front(), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn cond(&self) -> Option<Node> {
        None
    }

    fn set_next(&mut self, node: Box<dyn GraphNode>) -> &mut dyn GraphNode {
        self.next_node = Some(node);
        &mut **self.next_node.as_mut().unwrap()
    }

    fn name(&self) -> &'static str {
        "iterate reads from two fastq files"
    }
}

/// Create a read iterator over fastq records from a file.
///
/// Larger `chunk_size` uses more memory, but reduces the overhead of allocations, multithreading,
/// etc.
#[must_use]
pub fn iter_fastq1(file: impl AsRef<str>) -> Result<Box<dyn GraphNode + 'static>> {
    let reader = Mutex::new(parse_fastx_file(file.as_ref()).map_err(|e| Error::FileIo {
        file: file.as_ref().to_owned(),
        source: Box::new(e),
    })?);
    Ok(Box::new(Fastq1Node::<'static> {
        next_node: None,
        reader,
        buf: ReadBuf::new(),
        origin: Arc::new(Origin::File(file.as_ref().to_owned())),
        idx: AtomicUsize::new(0),
        interleaved: false,
    }))
}

/// Create a read iterator over interleaved paired-end fastq records from a file.
///
/// Larger `chunk_size` uses more memory, but reduces the overhead of allocations, multithreading,
/// etc.
#[must_use]
pub fn iter_fastq_interleaved(
    file: impl AsRef<str>,
) -> Result<Box<dyn GraphNode + 'static>> {
    let reader = Mutex::new(parse_fastx_file(file.as_ref()).map_err(|e| Error::FileIo {
        file: file.as_ref().to_owned(),
        source: Box::new(e),
    })?);
    Ok(Box::new(Fastq1Node::<'static> {
        next_node: None,
        reader,
        buf: ReadBuf::new(),
        origin: Arc::new(Origin::File(file.as_ref().to_owned())),
        idx: AtomicUsize::new(0),
        interleaved: true,
    }))
}

/// Create a read iterator over paired-end fastq records from two different files.
///
/// Larger `chunk_size` uses more memory, but reduces the overhead of allocations, multithreading,
/// etc.
#[must_use]
pub fn iter_fastq2(
    file1: impl AsRef<str>,
    file2: impl AsRef<str>,
) -> Result<Box<dyn GraphNode>> {
    let reader1 = Mutex::new(parse_fastx_file(file1.as_ref()).map_err(|e| Error::FileIo {
        file: file1.as_ref().to_owned(),
        source: Box::new(e),
    })?);
    let reader2 = Mutex::new(parse_fastx_file(file2.as_ref()).map_err(|e| Error::FileIo {
        file: file2.as_ref().to_owned(),
        source: Box::new(e),
    })?);
    Ok(Box::new(Fastq2Node {
        next_node: None,
        reader1,
        reader2,
        buf: ReadBuf::new(),
        origin1: Arc::new(Origin::File(file1.as_ref().to_owned())),
        origin2: Arc::new(Origin::File(file2.as_ref().to_owned())),
        idx: AtomicUsize::new(0),
    }))
}

/// Create a read iterator over fastq records from a byte slice.
#[must_use]
pub fn iter_fastq1_bytes<'reader>(bytes: &'reader [u8]) -> Result<Box<dyn GraphNode + 'reader>> {
    let reader = Mutex::new(parse_fastx_reader(bytes).map_err(|e| Error::BytesIo(Box::new(e)))?);
    Ok(Box::new(Fastq1Node::<'reader> {
        next_node: None,
        reader,
        buf: ReadBuf::new(),
        origin: Arc::new(Origin::Bytes),
        idx: AtomicUsize::new(0),
        interleaved: false,
    }))
}

/// Create a read iterator over interleaved paired-end fastq records from a byte slice.
#[must_use]
pub fn iter_fastq_interleaved_bytes<'reader>(bytes: &'reader [u8]) -> Result<Box<dyn GraphNode + 'reader>> {
    let reader = Mutex::new(parse_fastx_reader(bytes).map_err(|e| Error::BytesIo(Box::new(e)))?);
    Ok(Box::new(Fastq1Node::<'reader> {
        next_node: None,
        reader,
        buf: ReadBuf::new(),
        origin: Arc::new(Origin::Bytes),
        idx: AtomicUsize::new(0),
        interleaved: true,
    }))
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
