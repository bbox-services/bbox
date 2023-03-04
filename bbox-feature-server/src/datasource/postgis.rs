use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Result;

#[derive(Clone)]
pub struct PgDatasource(PgPool);

impl PgDatasource {
    pub async fn new_pool(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(8)
            .connect(url)
            .await?;
        Ok(PgDatasource(pool))
    }
}
