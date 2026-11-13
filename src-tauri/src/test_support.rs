//! Helpers shared by unit tests.

use std::path::PathBuf;

/// Reads a redacted fixture from `<repo>/fixtures/<name>`.
pub fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}
