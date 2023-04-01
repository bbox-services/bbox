use crate::writer::TileWriter;
use crate::SeedArgs;
use async_trait::async_trait;
use log::debug;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use std::env;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct S3Writer {
    bucket: String,
    region: rusoto_core::Region,
}

#[async_trait]
impl TileWriter for S3Writer {
    fn from_args(args: &SeedArgs) -> anyhow::Result<Self> {
        let bucket = match args.s3_path.as_ref().unwrap().strip_prefix("s3://") {
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

        Ok(S3Writer { bucket, region })
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

impl S3Writer {
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
