use actix_web::http::header::HeaderMap;
use flate2::{read::GzEncoder, Compression as GzCompression};
use std::io::Read;

#[derive(Clone, PartialEq, Debug)]
pub enum Compression {
    // Unknown,
    None,
    Gzip,
    // Brotli,
    // Zstd,
}

/// Tile reader response
pub struct TileResponse {
    pub content_type: Option<String>,
    pub headers: HeaderMap,
    pub body: Box<dyn Read + Send + Sync>,
}

impl TileResponse {
    pub fn new_headers() -> HeaderMap {
        HeaderMap::new()
    }
    /// Read tile body with optional compression
    pub fn read_bytes(mut self, compression: &Compression) -> Result<Vec<u8>, std::io::Error> {
        let mut bytes: Vec<u8> = Vec::new();
        match compression {
            Compression::Gzip => {
                let mut gz = GzEncoder::new(self.body, GzCompression::fast());
                gz.read_to_end(&mut bytes)?;
            }
            Compression::None => {
                self.body.read_to_end(&mut bytes)?;
            }
        }
        Ok(bytes)
    }
}
