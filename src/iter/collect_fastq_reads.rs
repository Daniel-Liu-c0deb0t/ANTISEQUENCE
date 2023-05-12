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
    file_writers: Mutex<FxHashMap<Vec<u8>, Arc<Mutex<dyn Write + std::marker::Send>>>>,
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
    fn next_chunk(&self) -> Vec<Read> {
        let reads = self.reads.next_chunk();
        let mut locked_writers = Vec::with_capacity(reads.len());

        {
            let mut file_writers = self.file_writers.lock().unwrap();

            let mut get_writer = |file_expr: &FormatExpr, read: &Read| {
                let file_name = file_expr.format(read, false);

                let locked_writer = file_writers.entry(file_name.clone()).or_insert_with(|| {
                    let file_path = std::str::from_utf8(&file_name).unwrap();
                    std::fs::create_dir_all(std::path::Path::new(file_path).parent().unwrap())
                        .unwrap();

                    let writer: Arc<Mutex<dyn Write + std::marker::Send>> =
                        if file_path.ends_with(".gz") {
                            Arc::new(Mutex::new(BufWriter::new(GzEncoder::new(
                                File::create(file_path).unwrap(),
                                Compression::default(),
                            ))))
                        } else {
                            Arc::new(Mutex::new(BufWriter::new(File::create(file_path).unwrap())))
                        };
                    writer
                });

                locked_writers.push(Arc::clone(locked_writer));
            };

            for read in reads.iter().filter(|r| self.selector_expr.matches(r)) {
                get_writer(&self.file_expr1, read);

                if let Some(file_expr2) = &self.file_expr2 {
                    get_writer(file_expr2, read);
                }
            }
        }

        if self.file_expr2.is_some() {
            for (locked_writer, read) in locked_writers
                .chunks(2)
                .zip(reads.iter().filter(|r| self.selector_expr.matches(r)))
            {
                let mut writer1 = locked_writer[0].lock().unwrap();
                let mut writer2 = locked_writer[1].lock().unwrap();
                let (record1, record2) = read.to_fastq2();
                write_fastq_record(&mut *writer1, record1);
                write_fastq_record(&mut *writer2, record2);
            }
        } else {
            for (locked_writer, read) in locked_writers
                .into_iter()
                .zip(reads.iter().filter(|r| self.selector_expr.matches(r)))
            {
                let mut writer = locked_writer.lock().unwrap();
                write_fastq_record(&mut *writer, read.to_fastq1());
            }
        }

        reads
    }

    fn finish(&self) {
        self.reads.finish();
    }
}
