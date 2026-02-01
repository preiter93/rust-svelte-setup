use crate::error::DBError;
use deadpool_postgres::Pool;
use std::fmt::Debug;
use tokio_postgres::Row;
use tonic::async_trait;
use uuid::Uuid;

use crate::proto::User;

#[async_trait]
pub trait DBClient: Send + Sync + 'static {
    async fn insert_user(&self, id: Uuid, name: &str, email: &str) -> Result<(), DBError>;

    async fn get_user(&self, id: Uuid) -> Result<User, DBError>;
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
    async fn insert_user(&self, id: Uuid, name: &str, email: &str) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO users (id, name, email) VALUES ($1, $2, $3)",
                &[&id, &name, &email],
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

        Ok(User::try_from(row)?)
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
    use rstest::rstest;
    use testutils::get_test_db;
    use tokio::sync::Mutex;
    use tonic::async_trait;
    use user::SERVICE_NAME;
    use uuid::Uuid;

    use super::*;

    use crate::error::DBError;

    pub struct MockDBClient {
        pub get_user: Mutex<Option<Result<User, DBError>>>,
        pub insert_user: Mutex<Option<Result<(), DBError>>>,
        pub get_user_id_from_oauth_id: Mutex<Option<Result<Uuid, DBError>>>,
    }
    impl Default for MockDBClient {
        fn default() -> Self {
            Self {
                insert_user: Mutex::new(None),
                get_user: Mutex::new(None),
                get_user_id_from_oauth_id: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl DBClient for MockDBClient {
        async fn insert_user(&self, _: Uuid, _: &str, _: &str) -> Result<(), DBError> {
            self.insert_user.lock().await.take().unwrap()
        }

        async fn get_user(&self, _: Uuid) -> Result<User, DBError> {
            self.get_user.lock().await.take().unwrap()
        }
    }

    #[derive(Clone)]
    struct DBUser {
        id: Uuid,
        name: &'static str,
        email: &'static str,
    }

    fn fixture_db_user<F>(mut func: F) -> DBUser
    where
        F: FnMut(&mut DBUser),
    {
        let mut user = DBUser {
            id: fixture_uuid(),
            name: "name",
            email: "email",
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
        let pool = get_test_db(SERVICE_NAME, migrations)
            .await
            .expect("failed to get connection to test db");
        let db_client = PostgresDBClient { pool };

        for user in given_user {
            db_client
                .insert_user(user.id.clone(), &user.name, &user.email)
                .await
                .expect("failed to insert user");
        }

        test_fn(db_client).await;
    }

    #[rstest]
    #[case::happy_path(
        fixture_uuid(),
        vec![fixture_db_user(|_| {})],
        Ok(fixture_user(|_| {}))
    )]
    #[case::not_found(
        Uuid::parse_str("99999999-9999-9999-9999-999999999999").unwrap(),
        vec![],
        Err(DBError::NotFound)
    )]
    #[tokio::test]
    async fn test_get_user(
        #[case] user_id: Uuid,
        #[case] given_users: Vec<DBUser>,
        #[case] want: Result<User, DBError>,
    ) {
        run_db_test(given_users, |db_client| async move {
            let got = db_client.get_user(user_id).await;

            match (got, want) {
                (Ok(got_user), Ok(want_user)) => assert_eq!(got_user, want_user),
                (Err(got_err), Err(want_err)) => {
                    assert_eq!(format!("{got_err}"), format!("{want_err}"))
                }
                (got, want) => panic!("expected {want:?}, got {got:?}"),
            }
        })
        .await;
    }
}
