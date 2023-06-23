use crate::cache::{BoxRead, TileCacheError, TileCacheType, TileReader, TileWriter};
use crate::cli::SeedArgs;
use crate::config::S3CacheCfg;
use async_trait::async_trait;
use bbox_core::endpoints::TileResponse;
use log::debug;
use rusoto_s3::{PutObjectError, PutObjectRequest, S3Client, S3};
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use tile_grid::Xyz;

#[derive(Clone, Debug)]
pub struct S3Cache {
    bucket: String,
    region: rusoto_core::Region,
}

#[derive(thiserror::Error, Debug)]
pub enum S3CacheError {
    #[error("S3 path should be 's3://bucket'")]
    InvalidS3Path,
    #[error("Reading input failed: {0}")]
    ReadInputError(#[source] std::io::Error),
    #[error("Upload failed: {0}")]
    UploadFailed(#[source] rusoto_core::RusotoError<PutObjectError>),
}

impl S3Cache {
    pub fn from_s3_path(s3_path: &str) -> Result<Self, S3CacheError> {
        let bucket = match s3_path.strip_prefix("s3://") {
            None => return Err(S3CacheError::InvalidS3Path),
            Some(bucket) => {
                if bucket.contains('/') {
                    return Err(S3CacheError::InvalidS3Path);
                } else {
                    bucket.to_string()
                }
            }
        };
        let region = match env::var("S3_ENDPOINT_URL") {
            Ok(endpoint) => rusoto_core::Region::Custom {
                name: "region".to_string(),
                endpoint,
            },
            Err(_) => rusoto_core::Region::default(),
        };

        Ok(S3Cache { bucket, region })
    }
    pub fn from_config(cfg: &S3CacheCfg) -> Result<Self, TileCacheError> {
        Self::from_s3_path(&cfg.path).map_err(Into::into)
    }
}

impl TileCacheType for S3Cache {
    fn from_args(args: &SeedArgs) -> Result<Self, TileCacheError> {
        Self::from_s3_path(args.s3_path.as_ref().unwrap()).map_err(Into::into)
    }
}

#[async_trait]
impl TileWriter for S3Cache {
    async fn put_tile(&self, key: String, mut input: BoxRead) -> Result<(), TileCacheError> {
        let bucket = self.bucket.clone();
        let client = S3Client::new(self.region.clone());
        let mut data = Vec::with_capacity(4096);
        let content_length = match input.read_to_end(&mut data) {
            Ok(len) => len as i64,
            Err(e) => return Err(S3CacheError::ReadInputError(e).into()),
        };
        debug!("cp {key} ({content_length} bytes)");

        if let Err(e) = {
            let request = PutObjectRequest {
                bucket,
                key,
                body: Some(data.into()),
                content_length: Some(content_length),
                ..Default::default()
            };
            client.put_object(request).await
        } {
            eprintln!("Upload failed: {e}");
            return Err(S3CacheError::UploadFailed(e).into());
        }
        Ok(())
    }
}

impl S3Cache {
    /// Put tile from temporary file
    pub async fn put_file(&self, base_dir: &PathBuf, path: String) -> Result<(), TileCacheError> {
        let mut fullpath = base_dir.clone();
        fullpath.push(&path);
        let p = fullpath.as_path();
        let reader = Box::new(BufReader::new(
            File::open(p).map_err(|e| TileCacheError::FileError(fullpath.clone(), e))?,
        ));
        self.put_tile(path, reader).await?;
        fs::remove_file(p).map_err(|e| TileCacheError::FileError(fullpath.clone(), e))?;

        Ok(())
    }
}

impl TileReader for S3Cache {
    fn get_tile(&self, _tile: &Xyz, _format: &str) -> Option<TileResponse> {
        // 2nd level cache lookup is not supported
        None
    }
}
