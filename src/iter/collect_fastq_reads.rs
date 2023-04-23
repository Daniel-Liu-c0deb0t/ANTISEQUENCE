use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Mutex, RwLock};

use rustc_hash::FxHashMap;

use flate2::{write::GzEncoder, Compression};

use crate::fastq::*;
use crate::iter::*;

pub struct CollectFastqReads<R: Reads> {
    reads: R,
    selector_expr: SelectorExpr,
    file_expr: FormatExpr,
    file_writers: RwLock<FxHashMap<String, Mutex<Box<dyn Write + std::marker::Send>>>>,
}

impl<R: Reads> CollectFastqReads<R> {
    pub fn new(reads: R, selector_expr: SelectorExpr, file_expr: FormatExpr) -> Self {
        Self {
            reads,
            selector_expr,
            file_expr,
            file_writers: RwLock::new(FxHashMap::default()),
        }
    }
}

impl<R: Reads> Reads for CollectFastqReads<R> {
    fn next_chunk(&self) -> Vec<Read> {
        let reads = self.reads.next_chunk();

        {
            let mut file_writers = self.file_writers.write().unwrap();

            for read in reads.iter().filter(|r| self.selector_expr.matches(r)) {
                let file_name = self.file_expr.format(read);

                file_writers.entry(file_name.clone()).or_insert_with(|| {
                    std::fs::create_dir_all(std::path::Path::new(&file_name).parent().unwrap())
                        .unwrap();

                    let writer: Box<dyn Write + std::marker::Send> = if file_name.ends_with(".gz") {
                        Box::new(BufWriter::new(GzEncoder::new(
                            File::create(&file_name).unwrap(),
                            Compression::default(),
                        )))
                    } else {
                        Box::new(BufWriter::new(File::create(&file_name).unwrap()))
                    };
                    Mutex::new(writer)
                });
            }
        }

        for read in reads.iter().filter(|r| self.selector_expr.matches(r)) {
            let file_name = self.file_expr.format(read);

            let file_writers = self.file_writers.read().unwrap();
            let mut writer = file_writers[&file_name].lock().unwrap();
            write_fastq_record(&mut *writer, read.to_fastq());
        }

        reads
    }
}
