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

#[cfg(test)]
pub mod test {
    use crate::{
        proto::User,
        utils::test::{fixture_user, fixture_uuid},
    };
    use shared::test_utils::get_test_db;
    use tokio::sync::Mutex;
    use tonic::async_trait;
    use user::SERVICE_NAME;
    use uuid::Uuid;

    use super::*;

    use crate::error::DBError;

    pub struct MockDBClient {
        pub get_user: Mutex<Option<Result<User, DBError>>>,
        pub insert_user: Mutex<Option<Result<(), DBError>>>,
        pub get_user_id_from_google_id: Mutex<Option<Result<Uuid, DBError>>>,
    }
    impl Default for MockDBClient {
        fn default() -> Self {
            Self {
                insert_user: Mutex::new(None),
                get_user: Mutex::new(None),
                get_user_id_from_google_id: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl DBClient for MockDBClient {
        async fn insert_user(&self, _: Uuid, _: &str, _: &str, _: &str) -> Result<(), DBError> {
            self.insert_user.lock().await.take().unwrap()
        }

        async fn get_user(&self, _: Uuid) -> Result<User, DBError> {
            self.get_user.lock().await.take().unwrap()
        }

        async fn get_user_id_from_google_id(&self, _: &str) -> Result<Uuid, DBError> {
            self.get_user_id_from_google_id.lock().await.take().unwrap()
        }
    }

    struct DBUser {
        id: Uuid,
        name: &'static str,
        email: &'static str,
        google_id: &'static str,
    }

    fn fixture_db_user<F>(mut func: F) -> DBUser
    where
        F: FnMut(&mut DBUser),
    {
        let mut user = DBUser {
            id: fixture_uuid(),
            name: "name",
            email: "email",
            google_id: "google-id",
        };
        func(&mut user);
        user
    }

    async fn run_db_test<F, Fut>(given_user: Vec<DBUser>, test_fn: F)
    where
        F: FnOnce(PostgresDBClient) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let migrations = std::fs::canonicalize("./migrations").unwrap();
        let db = get_test_db(SERVICE_NAME, migrations)
            .await
            .expect("failed to get test db client");
        let db_client = PostgresDBClient::new(db.pool.clone());

        for user in given_user {
            db_client
                .insert_user(user.id.clone(), &user.name, &user.email, &user.google_id)
                .await
                .expect("failed to insert user");
        }

        test_fn(db_client).await;
    }

    #[tokio::test]
    async fn test_get_user() {
        let user_id = fixture_uuid();
        let given_user = fixture_db_user(|u| u.id = user_id);
        let want_user = fixture_user(|u| u.id = user_id.to_string());

        run_db_test(vec![given_user], |db_client| async move {
            let got_user = db_client
                .get_user(user_id)
                .await
                .expect("failed to get user");

            assert_eq!(got_user, want_user);
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

        run_db_test(vec![], |db_client| async move {
            let got_result = db_client.get_user(user_id).await;

            assert!(matches!(got_result, Err(DBError::NotFound)));
        })
        .await;
    }
}
