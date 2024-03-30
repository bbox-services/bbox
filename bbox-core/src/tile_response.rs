use actix_web::http::header::{
    self, HeaderMap, HeaderValue, TryIntoHeaderPair, TryIntoHeaderValue,
};
use flate2::{read::GzEncoder, Compression as GzCompression};
use std::io::Read;

/// Tile data compression
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
    headers: HeaderMap,
    pub(crate) body: Box<dyn Read + Send + Sync>,
}

impl TileResponse {
    pub fn new() -> Self {
        TileResponse {
            headers: HeaderMap::new(),
            body: Box::new(std::io::empty()),
        }
    }
    /// Set response content type.
    pub fn set_content_type<V: TryIntoHeaderValue>(&mut self, value: V) -> &mut Self {
        if let Ok(value) = value.try_into_value() {
            self.headers.insert(header::CONTENT_TYPE, value);
        }
        self
    }
    /// Insert a header, replacing any that were set with an equivalent field name.
    pub fn insert_header(&mut self, header: impl TryIntoHeaderPair) -> &mut Self {
        if let Ok((key, value)) = header.try_into_pair() {
            self.headers.insert(key, value);
        }
        self
    }
    pub fn set_headers(&mut self, headers: &HeaderMap) -> &mut Self {
        for (key, value) in headers {
            self.insert_header((key, value));
        }
        self
    }
    pub fn with_body(mut self, body: Box<dyn Read + Send + Sync>) -> TileResponse {
        self.body = body;
        self
    }
    pub fn content_type(&self) -> Option<&HeaderValue> {
        self.headers.get(header::CONTENT_TYPE)
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
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
