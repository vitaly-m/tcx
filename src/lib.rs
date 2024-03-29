#[macro_use]
extern crate lazy_static;

use std::io::BufRead;

use quick_xml::Reader;

pub use read::ReadError;
pub use types::*;

mod read;
mod types;

/// Read the content of TCX xml data into TrainingCenterDatabase structure
/// ```
/// let tcx_bytes: &[u8] = include_bytes!("../test_resources/+__2020-12-28_16-36-16.TCX.xml");
/// assert_eq!(true, tcx::read(tcx_bytes).is_ok());
/// ```
pub fn read<B: BufRead>(buf_reader: B) -> Result<TrainingCenterDatabase, ReadError> {
    let mut reader = Reader::from_reader(buf_reader);
    read::read_training_center(&mut reader)
}
