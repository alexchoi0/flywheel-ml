pub mod entity;
pub mod migration;
pub mod repo;

pub use entity::*;
pub use repo::*;

use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct Database {
    conn: DatabaseConnection,
}

impl Database {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub async fn connect(database_url: &str) -> Result<Self, sea_orm::DbErr> {
        let conn = sea_orm::Database::connect(database_url).await?;
        Ok(Self { conn })
    }
}
