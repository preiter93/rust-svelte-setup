use crate::error::DBError;
use deadpool_postgres::Pool;
use std::fmt::Debug;
use tokio_postgres::Row;
use tonic::async_trait;
use uuid::Uuid;

use crate::proto::User;

#[async_trait]
pub trait DBClient: Send + Sync + 'static {
    async fn insert_user(
        &self,
        id: Uuid,
        name: &str,
        email: &str,
        google_id: &str,
    ) -> Result<(), DBError>;

    async fn get_user(&self, id: Uuid) -> Result<User, DBError>;

    async fn get_user_id_from_google_id(&self, google_id: &str) -> Result<Uuid, DBError>;
}

#[derive(Clone, Debug)]
pub struct PostgresDBClient {
    pub pool: Pool,
}

impl PostgresDBClient {
    #[must_use]
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DBClient for PostgresDBClient {
    /// # Errors
    /// - if the database connection cannot be established
    /// - if the database query fails
    async fn insert_user(
        &self,
        id: Uuid,
        name: &str,
        email: &str,
        google_id: &str,
    ) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO users (id, name, email, google_id) VALUES ($1, $2, $3, $4)",
                &[&id, &name, &email, &google_id],
            )
            .await?;

        Ok(())
    }

    /// # Errors
    /// - if the database connection cannot be established
    /// - if the database query fails
    /// - If the user is not found
    async fn get_user(&self, id: Uuid) -> Result<User, DBError> {
        let client = self.pool.get().await?;

        let stmt = client
            .prepare("SELECT id, name, email FROM users WHERE id = $1")
            .await?;
        let row = client.query_opt(&stmt, &[&id]).await?;
        let Some(row) = row else {
            return Err(DBError::NotFound);
        };

        let user: User = User::try_from(row)?;

        Ok(user)
    }

    /// # Errors
    /// - if the database connection cannot be established
    /// - if the database query fails
    /// - If the user is not found
    async fn get_user_id_from_google_id(&self, google_id: &str) -> Result<Uuid, DBError> {
        let client = self.pool.get().await?;

        let stmt = client
            .prepare("SELECT id FROM users WHERE google_id = $1")
            .await?;
        let row = client.query_opt(&stmt, &[&google_id]).await?;
        let Some(row) = row else {
            return Err(DBError::NotFound);
        };

        let id: Uuid = row.try_get("id")?;

        Ok(id)
    }
}

impl TryFrom<Row> for User {
    type Error = DBError;

    fn try_from(value: Row) -> Result<Self, DBError> {
        let id: Uuid = value.try_get("id")?;
        let name: String = value.try_get("name")?;
        let email: String = value.try_get("email")?;

        Ok(User {
            id: id.to_string(),
            name,
            email,
        })
    }
}
