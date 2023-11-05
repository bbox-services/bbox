use crate::cli::SeedArgs;
use crate::config::S3StoreCfg;
use crate::store::{BoxRead, TileReader, TileStoreError, TileStoreType, TileWriter};
use async_trait::async_trait;
use bbox_core::endpoints::TileResponse;
use log::debug;
use rusoto_s3::{PutObjectError, PutObjectRequest, S3Client, S3};
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use tile_grid::Xyz;

#[derive(Clone, Debug)]
pub struct S3Store {
    bucket: String,
    region: rusoto_core::Region,
}

#[derive(thiserror::Error, Debug)]
pub enum S3StoreError {
    #[error("S3 path should be 's3://bucket'")]
    InvalidS3Path,
    #[error("Reading input failed: {0}")]
    ReadInputError(#[source] std::io::Error),
    #[error("Upload failed: {0}")]
    UploadFailed(#[source] rusoto_core::RusotoError<PutObjectError>),
}

impl S3Store {
    pub fn from_s3_path(s3_path: &str) -> Result<Self, S3StoreError> {
        let bucket = match s3_path.strip_prefix("s3://") {
            None => return Err(S3StoreError::InvalidS3Path),
            Some(bucket) => {
                if bucket.contains('/') {
                    return Err(S3StoreError::InvalidS3Path);
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

        Ok(S3Store { bucket, region })
    }
    pub fn from_config(cfg: &S3StoreCfg) -> Result<Self, TileStoreError> {
        Self::from_s3_path(&cfg.path).map_err(Into::into)
    }
}

impl TileStoreType for S3Store {
    fn from_args(args: &SeedArgs) -> Result<Self, TileStoreError> {
        Self::from_s3_path(args.s3_path.as_ref().unwrap()).map_err(Into::into)
    }
}

#[async_trait]
impl TileWriter for S3Store {
    async fn put_tile(&self, key: String, mut input: BoxRead) -> Result<(), TileStoreError> {
        let bucket = self.bucket.clone();
        let client = S3Client::new(self.region.clone());
        let mut data = Vec::with_capacity(4096);
        let content_length = match input.read_to_end(&mut data) {
            Ok(len) => len as i64,
            Err(e) => return Err(S3StoreError::ReadInputError(e).into()),
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
            return Err(S3StoreError::UploadFailed(e).into());
        }
        Ok(())
    }
}

impl S3Store {
    /// Put tile from temporary file
    pub async fn put_file(&self, base_dir: &Path, path: String) -> Result<(), TileStoreError> {
        let mut fullpath = base_dir.to_path_buf();
        fullpath.push(&path);
        let p = fullpath.as_path();
        let reader = Box::new(BufReader::new(
            File::open(p).map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?,
        ));
        self.put_tile(path, reader).await?;
        fs::remove_file(p).map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?;

        Ok(())
    }
}

impl TileReader for S3Store {
    fn exists(&self, _path: &str) -> bool {
        // 2nd level cache lookup is not supported
        false
    }
    fn get_tile(&self, _tile: &Xyz, _format: &str) -> Option<TileResponse> {
        // 2nd level cache lookup is not supported
        None
    }
}
