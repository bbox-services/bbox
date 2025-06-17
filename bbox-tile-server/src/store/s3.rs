use crate::config::{S3StoreCfg, StoreCompressionCfg};
use crate::store::{
    CacheLayout, NoStore, StoreFromConfig, TileReader, TileStore, TileStoreError, TileWriter,
};
use async_trait::async_trait;
use bbox_core::{Compression, Format, TileResponse};
use log::debug;
use martin_mbtiles::Metadata;
use rusoto_s3::{PutObjectError, PutObjectRequest, S3Client, S3};
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use tile_grid::Xyz;

#[derive(Clone, Debug)]
pub struct S3Store {
    bucket: String,
    region: rusoto_core::Region,
    compression: StoreCompressionCfg,
    format: Format,
}

#[derive(thiserror::Error, Debug)]
pub enum S3StoreError {
    #[error("S3 path should be 's3://bucket'")]
    InvalidS3Path,
    #[error("Reading input failed: {0}")]
    ReadInputError(#[source] std::io::Error),
    #[error("Upload failed: {0}")]
    UploadFailed(#[source] Box<rusoto_core::RusotoError<PutObjectError>>),
}

impl StoreFromConfig for S3StoreCfg {
    fn to_store(
        &self,
        _tileset_name: &str,
        format: &Format,
        compression: &Option<StoreCompressionCfg>,
        _metadata: Metadata,
    ) -> Box<dyn TileStore> {
        let store = S3Store::from_s3_path(&self.path, compression, *format).unwrap();
        Box::new(store)
    }
}

impl S3Store {
    pub fn from_s3_path(
        s3_path: &str,
        compression: &Option<StoreCompressionCfg>,
        format: Format,
    ) -> Result<Self, S3StoreError> {
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
        let compression = compression.clone().unwrap_or(StoreCompressionCfg::None);

        Ok(S3Store {
            bucket,
            region,
            compression,
            format,
        })
    }
}

#[async_trait]
impl TileStore for S3Store {
    fn compression(&self) -> Compression {
        match self.compression {
            StoreCompressionCfg::Gzip => Compression::Gzip,
            StoreCompressionCfg::None => Compression::None,
        }
    }
    async fn setup_reader(&self, seeding: bool) -> Result<Box<dyn TileReader>, TileStoreError> {
        if seeding {
            Ok(Box::new(self.clone()))
        } else {
            Ok(Box::new(NoStore))
        }
    }
    async fn setup_writer(&self, seeding: bool) -> Result<Box<dyn TileWriter>, TileStoreError> {
        if seeding {
            Ok(Box::new(self.clone()))
        } else {
            Ok(Box::new(NoStore))
        }
    }
}

#[async_trait]
impl TileWriter for S3Store {
    async fn exists(&self, _xyz: &Xyz) -> bool {
        // 2nd level cache lookup is not supported
        false
    }
    async fn put_tile(&self, xyz: &Xyz, data: Vec<u8>) -> Result<(), TileStoreError> {
        let key = CacheLayout::Zxy.path_string(&PathBuf::new(), xyz, &self.format);
        self.put_data(key, data).await
    }
}

impl S3Store {
    pub async fn put_data(&self, key: String, data: Vec<u8>) -> Result<(), TileStoreError> {
        let bucket = self.bucket.clone();
        // TODO: Workaround for https://github.com/rusoto/rusoto/issues/1980
        let client = S3Client::new(self.region.clone());
        let content_length = data.len() as i64;
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
            return Err(S3StoreError::UploadFailed(Box::new(e)).into());
        }
        Ok(())
    }
    /// Put tile from temporary file
    #[allow(dead_code)]
    pub async fn copy_tile(&self, base_dir: &Path, xyz: &Xyz) -> Result<(), TileStoreError> {
        let fullpath = CacheLayout::Zxy.path(base_dir, xyz, &self.format);
        let p = fullpath.as_path();
        let mut data =
            BufReader::new(File::open(p).map_err(|e| TileStoreError::FileError(p.into(), e))?);
        let mut bytes = Vec::new();
        data.read_to_end(&mut bytes)?;
        self.put_tile(xyz, bytes).await?;
        fs::remove_file(p).map_err(|e| TileStoreError::FileError(p.into(), e))?;

        Ok(())
    }
}

#[async_trait]
impl TileReader for S3Store {
    async fn get_tile(&self, _xyz: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        // 2nd level cache lookup is not supported
        Ok(None)
    }
}
