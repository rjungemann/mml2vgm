//! MML compiler module
//!
//! This module contains the MML parsing and compilation logic.
//! 
//! # Submodules
//! - `ast`: Abstract Syntax Tree definitions
//! - `lexer`: Tokenizer for MML source code
//! - `parser`: Recursive descent parser
//! - `sema`: Semantic analysis (stub)
//! - `codegen`: Code generation for VGM/XGM/ZGM formats

pub mod ast;
pub mod codegen;
pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod sample_resolver;
pub mod sema;

#[cfg(test)]
mod tests;
