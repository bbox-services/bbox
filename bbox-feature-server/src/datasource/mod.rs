use crate::datasource::gpkg::SqliteConnections;
use crate::datasource::postgis::PgConnections;
use sqlx::Result;
use std::collections::HashMap;

pub mod gpkg;
pub mod postgis;

#[derive(Clone, Debug)]
pub struct DsConnections {
    sqlite_connections: HashMap<String, SqliteConnections>,
    pg_connections: HashMap<String, PgConnections>,
}

impl DsConnections {
    pub fn new() -> Self {
        DsConnections {
            sqlite_connections: HashMap::new(),
            pg_connections: HashMap::new(),
        }
    }
    pub async fn add_sqlite_pool(&mut self, gpkg: &str) -> Result<()> {
        let pool = SqliteConnections::new_pool(gpkg).await?;
        self.sqlite_connections.insert(gpkg.to_string(), pool);
        Ok(())
    }
    pub fn sqlite_pool(&self, gpkg: &str) -> Option<&SqliteConnections> {
        self.sqlite_connections.get(gpkg)
    }
    pub async fn add_pg_pool(&mut self, url: &str) -> Result<()> {
        let pool = PgConnections::new_pool(url).await?;
        self.pg_connections.insert(url.to_string(), pool);
        Ok(())
    }
    /// Close all connections
    pub async fn reset_pool(&mut self) -> Result<()> {
        for (_, _pool) in &self.sqlite_connections {
            //TODO
        }
        Ok(())
    }
}
