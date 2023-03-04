use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Result;

#[derive(Clone, Debug)]
pub struct PgConnections(PgPool);

impl PgConnections {
    pub async fn new_pool(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(8)
            .connect(url)
            .await?;
        Ok(PgConnections(pool))
    }
}
