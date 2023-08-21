use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap;

use flate2::{write::GzEncoder, Compression};

use crate::graph::*;

pub struct OutputFastqNode {
    required_names: Vec<LabelOrAttr>,
    file_expr1: Expr,
    file_expr2: Option<Expr>,
    file_writers: Mutex<FxHashMap<Vec<u8>, Arc<Mutex<dyn Write + Send>>>>,
}

impl OutputFastqNode {
    const NAME: &'static str = "OutputFastqNode";

    /// Output reads (read 1 only) to a file whose path is specified by an expression.
    pub fn new1(file_expr: Expr) -> Self {
        Self {
            required_names: file_expr.required_names(),
            file_expr1: file_expr,
            file_expr2: None,
            file_writers: Mutex::new(FxHashMap::default()),
        }
    }

    /// Output read 1 and read 2 to two separate files whose paths are specified by expressions.
    ///
    /// The reads will be interleaved if the file path expressions are the same.
    pub fn new2(
        file_expr1: Expr,
        file_expr2: Expr,
    ) -> Self {
        let mut required_names = file_expr1.required_names();
        required_names.extend(file_expr2.required_names());

        Self {
            required_names,
            file_expr1,
            file_expr2: Some(file_expr2),
            file_writers: Mutex::new(FxHashMap::default()),
        }
    }

    // get the corresponding file writer for each read first so writing to different files can be parallelized
    fn get_writer(&self, file_name: &[u8]) -> std::io::Result<Arc<Mutex<dyn Write + Send>>> {
        use std::collections::hash_map::Entry::*;
        let mut file_writers = self.file_writers.lock().unwrap();

        match file_writers.entry(file_name.to_owned()) {
            Occupied(e) => {
                Ok(Arc::clone(e.get()))
            }
            Vacant(e) => {
                // need to create the output file
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

                Ok(Arc::clone(e.insert(writer)))
            }
        }
    }
}

impl GraphNode for OutputFastqNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let Some(read) = read else { panic!("Expected some read!") };

        let file_name1 =
            self.file_expr1
                .eval_bytes(&read, false)
                .map_err(|e| Error::NameError {
                    source: e,
                    read: read.clone(),
                    context: Self::NAME,
                })?;

        let locked_writer1 = self.get_writer(&file_name1).map_err(|e| Error::FileIo {
            file: utf8(&file_name1),
            source: Box::new(e),
        })?;

        if let Some(file_expr2) = &self.file_expr2 {
            let file_name2 = file_expr2
                    .eval_bytes(&read, false)
                    .map_err(|e| Error::NameError {
                        source: e,
                        read: read.clone(),
                        context: Self::NAME,
                    })?;

            let locked_writer2 = self.get_writer(&file_name2).map_err(|e| Error::FileIo {
                file: utf8(&file_name2),
                source: Box::new(e),
            })?;

            let (record1, record2) = read.to_fastq2().map_err(|e| Error::NameError {
                source: e,
                read: read.clone(),
                context: Self::NAME,
            })?;
            // interleave records if the same file is specified twice
            {
                let mut writer1 = locked_writer1.lock().unwrap();
                write_fastq_record(&mut *writer1, record1);
            }
            {
                let mut writer2 = locked_writer2.lock().unwrap();
                write_fastq_record(&mut *writer2, record2);
            }
        } else {
            let mut writer1 = locked_writer1.lock().unwrap();
            write_fastq_record(&mut *writer1, read.to_fastq1());
        }

        Ok((Some(read), false))
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &self.required_names
    }

    fn name(&self) -> &'static str {
        Self::NAME
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
