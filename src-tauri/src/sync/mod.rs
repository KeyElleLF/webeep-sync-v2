//! Sync engine: pure planning (`tree`, `extract`, `assign`, `paths`, `plan`) and I/O
//! (`local`, `download`, `execute`, `run`).

pub mod assign;
pub mod cleanup;
pub mod download;
pub mod execute;
pub mod extract;
pub mod folders;
pub mod links;
pub mod local;
pub mod paths;
pub mod plan;
pub mod retire;
pub mod run;
pub mod tree;
