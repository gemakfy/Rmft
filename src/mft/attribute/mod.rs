//! NTFS attributes module.
//!
//! Every file record in MFT stores its data (timestamps, names, content)
//! as a sequence of attributes starting at `first_attr_offset`.
//! The chain of attributes terminates with marker `0xFFFFFFFF`.

pub mod data;
pub mod filename;
pub mod header;
pub mod standard_info;

pub use data::DataAttribute;
pub use filename::{FILE_NAME_DATA_OFFSET, FileNameAttribute};
pub use header::{AttributeHeader, AttributeTypeCode};
pub use standard_info::{FILETIME_TO_UNIX_OFFSET, StandardInfoAttribute};
