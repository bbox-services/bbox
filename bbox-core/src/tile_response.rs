use actix_web::http::header::{
    self, HeaderMap, HeaderValue, TryIntoHeaderPair, TryIntoHeaderValue,
};
use flate2::{read::GzDecoder, read::GzEncoder, Compression as GzCompression};
use std::io::{Cursor, Read};

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

/// Tile response data
pub struct TileResponseData {
    headers: HeaderMap,
    pub body: Vec<u8>,
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
    /// Apply optional de-/compression
    pub fn with_compression(mut self, compression: &Compression) -> TileResponse {
        match (self.compression(), compression) {
            (Compression::None, Compression::Gzip) => {
                let gz = GzEncoder::new(self.body, GzCompression::fast());
                self.body = Box::new(gz);
                self.insert_header(("Content-Encoding", "gzip"));
            }
            (Compression::Gzip, Compression::None) => {
                let gz = GzDecoder::new(self.body);
                self.body = Box::new(gz);
                self.headers.remove(header::CONTENT_ENCODING);
            }
            _ => {}
        }
        self
    }
    pub fn content_type(&self) -> Option<&HeaderValue> {
        self.headers.get(header::CONTENT_TYPE)
    }
    pub fn compression(&self) -> Compression {
        match self.headers.get(header::CONTENT_ENCODING) {
            Some(v) if v == HeaderValue::from_static("gzip") => Compression::Gzip,
            _ => Compression::None,
        }
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    /// Read tile body with optional compression
    pub fn read_bytes(
        mut self,
        compression: &Compression,
    ) -> Result<TileResponseData, std::io::Error> {
        let mut response = TileResponseData {
            headers: self.headers,
            body: Vec::new(),
        };
        match compression {
            Compression::Gzip => {
                let mut gz = GzEncoder::new(self.body, GzCompression::fast());
                gz.read_to_end(&mut response.body)?;
                response.insert_header(("Content-Encoding", "gzip"));
            }
            Compression::None => {
                self.body.read_to_end(&mut response.body)?;
            }
        }
        Ok(response)
    }
}

impl Default for TileResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl TileResponseData {
    /// Insert a header, replacing any that were set with an equivalent field name.
    pub fn insert_header(&mut self, header: impl TryIntoHeaderPair) -> &mut Self {
        if let Ok((key, value)) = header.try_into_pair() {
            self.headers.insert(key, value);
        }
        self
    }
    pub fn compression(&self) -> Compression {
        match self.headers.get(header::CONTENT_ENCODING) {
            Some(v) if v == HeaderValue::from_static("gzip") => Compression::Gzip,
            _ => Compression::None,
        }
    }
    /// Read tile body with optional compression
    pub fn as_response(self, compression: &Compression) -> TileResponse {
        let mut response = TileResponse::new();
        response.set_headers(&self.headers);
        match (self.compression(), compression) {
            (Compression::None, Compression::Gzip) => {
                let gz = GzEncoder::new(Cursor::new(self.body), GzCompression::fast());
                response.body = Box::new(gz);
                response.insert_header(("Content-Encoding", "gzip"));
            }
            (Compression::Gzip, Compression::None) => {
                let gz = GzDecoder::new(Cursor::new(self.body));
                response.body = Box::new(gz);
                response.headers.remove(header::CONTENT_ENCODING);
            }
            _ => response.body = Box::new(Cursor::new(self.body)),
        }
        response
    }
}
