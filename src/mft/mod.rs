//! MFT parsing module.

pub mod attribute;
pub mod header;
pub mod record;

pub use header::{MftHeader, RECORD_SIZE};
pub use record::MftEntry;
