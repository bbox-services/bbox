use crate::cache::{TileReader, TileWriter};
use crate::cli::SeedArgs;
use async_trait::async_trait;
use log::debug;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Empty};
use std::path::PathBuf;
use tile_grid::Tile;

#[derive(Clone, Debug)]
pub struct S3Cache {
    bucket: String,
    region: rusoto_core::Region,
}

impl S3Cache {
    pub fn from_s3_path(s3_path: &str) -> anyhow::Result<Self> {
        let bucket = match s3_path.strip_prefix("s3://") {
            None => anyhow::bail!("S3 path has to start with 's3://'"),
            Some(bucket) => {
                if bucket.contains('/') {
                    anyhow::bail!("S3 path should be 's3://bucket'")
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
}

#[async_trait]
impl TileWriter for S3Cache {
    fn from_args(args: &SeedArgs) -> anyhow::Result<Self> {
        Self::from_s3_path(args.s3_path.as_ref().unwrap())
    }

    async fn put_tile(
        &self,
        key: String,
        mut input: Box<dyn std::io::Read + Send + Sync>,
    ) -> anyhow::Result<()> {
        let bucket = self.bucket.clone();
        let client = S3Client::new(self.region.clone());
        let mut data = Vec::with_capacity(4096);
        let content_length = match input.read_to_end(&mut data) {
            Ok(len) => len as i64,
            Err(e) => anyhow::bail!("Reading input failed: {e}"),
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
            eprintln!("Upload failed: {}", e);
            anyhow::bail!("Upload failed {e}");
        }
        Ok(())
    }
}

impl S3Cache {
    /// Put tile from temporary file
    pub async fn put_file(&self, base_dir: &PathBuf, path: String) -> anyhow::Result<()> {
        let mut fullpath = base_dir.clone();
        fullpath.push(&path);
        let p = fullpath.as_path();
        let reader = Box::new(BufReader::new(File::open(p)?));
        self.put_tile(path, reader).await?;
        fs::remove_file(p)?;

        Ok(())
    }
}

impl TileReader<Empty> for S3Cache {
    /// 2nd level cache lookup is not supported
    fn get_tile(&self, _tile: &Tile, _format: &str) -> Option<Empty> {
        None
    }
}
