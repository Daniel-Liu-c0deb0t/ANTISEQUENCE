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
    file_expr: FormatExpr,
    file_writers: Mutex<FxHashMap<String, Arc<Mutex<dyn Write + std::marker::Send>>>>,
}

impl<R: Reads> CollectFastqReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr, file_expr: FormatExpr) -> Self {
        Self {
            reads,
            selector_expr,
            file_expr,
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

            for read in reads.iter().filter(|r| self.selector_expr.matches(r)) {
                let file_name = self.file_expr.format(read, false);

                let locked_writer = file_writers.entry(file_name.clone()).or_insert_with(|| {
                    std::fs::create_dir_all(std::path::Path::new(&file_name).parent().unwrap())
                        .unwrap();

                    let writer: Arc<Mutex<dyn Write + std::marker::Send>> =
                        if file_name.ends_with(".gz") {
                            Arc::new(Mutex::new(BufWriter::new(GzEncoder::new(
                                File::create(&file_name).unwrap(),
                                Compression::default(),
                            ))))
                        } else {
                            Arc::new(Mutex::new(BufWriter::new(
                                File::create(&file_name).unwrap(),
                            )))
                        };
                    writer
                });

                locked_writers.push(Arc::clone(locked_writer));
            }
        }

        for (locked_writer, read) in locked_writers
            .into_iter()
            .zip(reads.iter().filter(|r| self.selector_expr.matches(r)))
        {
            let mut writer = locked_writer.lock().unwrap();
            write_fastq_record(&mut *writer, read.to_fastq());
        }

        reads
    }
}
