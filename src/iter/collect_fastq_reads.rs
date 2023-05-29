use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap;

use flate2::{write::GzEncoder, Compression};

use crate::fastq::*;
use crate::iter::*;

pub struct CollectFastqReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    file_expr1: FormatExpr,
    file_expr2: Option<FormatExpr>,
    file_writers: Mutex<FxHashMap<Vec<u8>, Arc<Mutex<dyn Write + Send>>>>,
}

impl<R: Reads> CollectFastqReads<R> {
    pub fn new1(reads: R, selector_expr: SelectorExpr, file_expr: FormatExpr) -> Self {
        Self {
            reads,
            selector_expr,
            file_expr1: file_expr,
            file_expr2: None,
            file_writers: Mutex::new(FxHashMap::default()),
        }
    }

    pub fn new2(
        reads: R,
        selector_expr: SelectorExpr,
        file_expr1: FormatExpr,
        file_expr2: FormatExpr,
    ) -> Self {
        Self {
            reads,
            selector_expr,
            file_expr1,
            file_expr2: Some(file_expr2),
            file_writers: Mutex::new(FxHashMap::default()),
        }
    }
}

impl<R: Reads> Reads for CollectFastqReads<R> {
    fn next_chunk(&self) -> Result<Vec<Read>> {
        let reads = self.reads.next_chunk()?;
        let mut locked_writers = Vec::with_capacity(reads.len());

        {
            let mut file_writers = self.file_writers.lock().unwrap();

            let mut get_writer = |file_name: &[u8]| -> std::io::Result<()> {
                use std::collections::hash_map::Entry::*;
                match file_writers.entry(file_name.to_owned()) {
                    Occupied(e) => {
                        locked_writers.push(Arc::clone(e.get()));
                    }
                    Vacant(e) => {
                        let file_path = std::str::from_utf8(file_name).unwrap();

                        if let Some(parent) = std::path::Path::new(file_path).parent() {
                            std::fs::create_dir_all(parent)?;
                        }

                        let writer: Arc<Mutex<dyn Write + Send>> = if file_path.ends_with(".gz") {
                            Arc::new(Mutex::new(BufWriter::new(GzEncoder::new(
                                File::create(file_path)?,
                                Compression::default(),
                            ))))
                        } else {
                            Arc::new(Mutex::new(BufWriter::new(File::create(file_path)?)))
                        };
                        locked_writers.push(Arc::clone(e.insert(writer)));
                    }
                }

                Ok(())
            };

            for read in reads.iter() {
                if !(self
                    .selector_expr
                    .matches(read)
                    .map_err(|e| Error::NameError {
                        source: e,
                        read: read.clone(),
                        context: "collecting into fastq file(s)",
                    })?)
                {
                    continue;
                }

                let file_name =
                    self.file_expr1
                        .format(read, false)
                        .map_err(|e| Error::NameError {
                            source: e,
                            read: read.clone(),
                            context: "collecting into fastq file(s)",
                        })?;
                get_writer(&file_name).map_err(|e| Error::FileIo {
                    file: utf8(&file_name),
                    source: Box::new(e),
                })?;

                if let Some(file_expr2) = &self.file_expr2 {
                    let file_name =
                        file_expr2
                            .format(read, false)
                            .map_err(|e| Error::NameError {
                                source: e,
                                read: read.clone(),
                                context: "collecting into fastq file(s)",
                            })?;
                    get_writer(&file_name).map_err(|e| Error::FileIo {
                        file: utf8(&file_name),
                        source: Box::new(e),
                    })?;
                }
            }
        }

        if self.file_expr2.is_some() {
            for (locked_writer, read) in locked_writers.chunks(2).zip(
                reads
                    .iter()
                    .filter(|r| self.selector_expr.matches(r).unwrap()),
            ) {
                let mut writer1 = locked_writer[0].lock().unwrap();
                let mut writer2 = locked_writer[1].lock().unwrap();
                let (record1, record2) = read.to_fastq2().map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: "collecting into fastq file(s)",
                })?;
                write_fastq_record(&mut *writer1, record1);
                write_fastq_record(&mut *writer2, record2);
            }
        } else {
            for (locked_writer, read) in locked_writers.into_iter().zip(
                reads
                    .iter()
                    .filter(|r| self.selector_expr.matches(r).unwrap()),
            ) {
                let mut writer = locked_writer.lock().unwrap();
                write_fastq_record(&mut *writer, read.to_fastq1());
            }
        }

        Ok(reads)
    }

    fn finish(self) -> Result<()> {
        self.reads.finish()
    }
}
