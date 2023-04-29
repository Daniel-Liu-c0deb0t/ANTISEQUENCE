use std::ops::RangeBounds;
use std::thread;

use crate::expr::*;
use crate::read::*;

pub mod trim_reads;
use trim_reads::*;

pub mod collect_fastq_reads;
use collect_fastq_reads::*;

pub mod for_each_reads;
use for_each_reads::*;

pub mod cut_reads;
use cut_reads::*;

pub mod set_reads;
use set_reads::*;

pub mod length_in_bounds_reads;
use length_in_bounds_reads::*;

pub mod retain_reads;
use retain_reads::*;

pub mod regex_match_reads;
use regex_match_reads::*;

pub trait Reads: Sized + std::marker::Sync {
    fn run(self, threads: usize) {
        assert!(threads >= 1);

        thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| while self.next_chunk().len() > 0 {});
            }
        });
    }

    #[must_use]
    fn for_each<F>(self, selector_expr: &str, func: F) -> ForEachReads<Self, F>
    where
        F: Fn(&mut Read) + std::marker::Sync,
    {
        ForEachReads::new(self, SelectorExpr::new(selector_expr), func)
    }

    #[must_use]
    fn length_in_bounds<B>(
        self,
        selector_expr: &str,
        attr: &str,
        bounds: B,
    ) -> LengthInBoundsReads<Self, B>
    where
        B: RangeBounds<usize> + std::marker::Sync,
    {
        LengthInBoundsReads::new(
            self,
            SelectorExpr::new(selector_expr),
            Attr::new(attr),
            bounds,
        )
    }

    #[must_use]
    fn cut(self, selector_expr: &str, transform_expr: &str, cut_idx: EndIdx) -> CutReads<Self> {
        CutReads::new(
            self,
            SelectorExpr::new(selector_expr),
            TransformExpr::new(transform_expr),
            cut_idx,
        )
    }

    #[must_use]
    fn trim<S>(self, selector_expr: &str, labels: impl AsRef<[S]>) -> TrimReads<Self>
    where
        S: AsRef<str>,
    {
        let labels = labels
            .as_ref()
            .iter()
            .map(|l| Label::new(l.as_ref()))
            .collect::<Vec<_>>();
        TrimReads::new(self, SelectorExpr::new(selector_expr), labels)
    }

    #[must_use]
    fn set(self, selector_expr: &str, label_or_attr: &str, format_expr: &str) -> SetReads<Self> {
        SetReads::new(
            self,
            SelectorExpr::new(selector_expr),
            LabelOrAttr::new(label_or_attr),
            FormatExpr::new(format_expr),
        )
    }

    #[must_use]
    fn regex_match(self, selector_expr: &str, attr: &str, regex: &str) -> RegexMatchReads<Self> {
        RegexMatchReads::new(
            self,
            SelectorExpr::new(selector_expr),
            Attr::new(attr),
            regex,
        )
    }

    #[must_use]
    fn collect_fastq1(self, selector_expr: &str, file_expr: &str) -> CollectFastqReads<Self> {
        CollectFastqReads::new1(
            self,
            SelectorExpr::new(selector_expr),
            FormatExpr::new(file_expr),
        )
    }

    #[must_use]
    fn collect_fastq2(
        self,
        selector_expr: &str,
        file_expr1: &str,
        file_expr2: &str,
    ) -> CollectFastqReads<Self> {
        CollectFastqReads::new2(
            self,
            SelectorExpr::new(selector_expr),
            FormatExpr::new(file_expr1),
            FormatExpr::new(file_expr2),
        )
    }

    #[must_use]
    fn retain(self, selector_expr: &str) -> RetainReads<Self> {
        RetainReads::new(self, SelectorExpr::new(selector_expr))
    }

    fn next_chunk(&self) -> Vec<Read>;
}
