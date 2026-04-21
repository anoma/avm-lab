//! AVM Core — the Anoma Virtual Machine implementation.
//!
//! This crate provides a faithful Rust implementation of the AVM specification,
//! an object-centric, message-passing, transactional virtual machine built on
//! interaction trees.
//!
//! # Architecture
//!
//! The crate is organized into layers that mirror the formal specification:
//!
//! - [`types`] — runtime values, identifiers, metadata
//! - [`itree`] — interaction trees (the program model)
//! - [`instruction`] — the layered instruction set architecture
//! - [`error`] — compositional error hierarchy
//! - [`trace`] — observability and event logging
//! - [`store`] — persistent object storage
//! - [`vm`] — VM state and runtime support
//! - [`interpreter`] — the execution engine

pub mod dsl;
pub mod error;
pub mod instruction;
pub mod interpreter;
pub mod itree;
pub mod store;
pub mod trace;
pub mod types;
pub mod vm;
