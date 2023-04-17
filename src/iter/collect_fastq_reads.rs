use crate::read::*;

pub struct CollectFastqReads<'r, R: Reads> {
    reads: &'r R,
    selector_expr: SelectorExpr,
    file_expr: FormatExpr,
    file_writers: RwLock<FxHashMap<String, Mutex<Box<dyn Writer>>>>,
}

impl<'r, R: Reads> CollectFastqReads<'r, R> {
    pub fn new(reads: &'r R, selector_expr: SelectorExpr, file_expr: FormatExpr) -> Self {
        Self {
            reads,
            selector_expr,
            file_expr,
        }
    }
}

impl<'r, R: Reads> Reads for CollectFastqReads<'r, R> {
    fn next_chunk(&self) -> Vec<Read> {
        let reads = self.reads.next_chunk();

        {
            let file_writers = self.file_writers.write().unwrap();

            for read in &reads {
                let file_name = self.file_expr.format(read);

                file_writers.entry(file_name.clone()).or_insert_with(|| {
                    let writer: Box<dyn Writer> = if file_name.ends_with(".gz") {
                        Box::new(BufWriter::new(GzipEncoder::new(File::create(&file_name).unwrap(), Compression::default())))
                    } else {
                        Box::new(BufWriter::new(File::create(&file_name).unwrap()))
                    };
                    Mutex::new(writer)
                });
            }
        }

        for read in &reads {
            let file_name = self.file_expr.format(read);

            let file_writers = self.file_writers.read().unwrap();
            let writer = file_writers[&file_name].lock().unwrap();
            write_fastq_record(&mut writer, read.to_fastq());
        }

        reads
    }
}
