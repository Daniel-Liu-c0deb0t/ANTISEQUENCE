# ANTISEQUENCE
Rust library for processing sequencing reads.

*Work in progress! Very early stages of the project.*

## Goals
* Robust, flexible, and actually universal primitives for manipulating raw DNA/RNA sequences from fastq files
* Blazing fast and scalable implementation using SIMD and CPU parallelization
* Simple interface allowing anyone (even non-Rustaceans) to pick up and use
* Extensible with custom Rust code and embeddable into existing pipelines

ANTISEQUENCE should enable you to build robust, efficient, and production-ready pipeline for your custom sequencing data.

## API
I am seeking feedback on the API design and improving error handling.

One question is whether to expose the API in Python or as a custom domain-specific language.
Having the API be fully in Rust is more efficient and retains the full power of a general programming language.
However, the Rust compiler is needed for development and it may be harder to directly embed in non-Rust projects.

## K-pop song
Enjoy a K-pop [song](https://youtu.be/pyf8cbqyfPs).
