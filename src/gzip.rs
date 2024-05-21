use std::io::Write;

use flate2::{write::GzEncoder, Compression};

pub struct Gzip(Vec<u8>);
impl Gzip {
    pub fn parse(s: &str) -> Self {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(s.as_bytes()).unwrap();
        let hex_str = e.finish().unwrap();
        Self(hex_str)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
