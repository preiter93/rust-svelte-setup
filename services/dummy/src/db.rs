use crate::error::DBError;
use deadpool_postgres::Pool;
use std::fmt::Debug;
use tokio_postgres::Row;
use tonic::async_trait;
use uuid::Uuid;

use crate::proto::Entity;

#[async_trait]
pub trait DBClient: Send + Sync + 'static {
    async fn insert_entity(&self, id: Uuid, user_id: Uuid) -> Result<(), DBError>;

    async fn get_entity(&self, id: Uuid, user_id: Uuid) -> Result<Entity, DBError>;
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
    async fn insert_entity(&self, id: Uuid, user_id: Uuid) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO entities (id, user_id) VALUES ($1, $2)",
                &[&id, &user_id],
            )
            .await?;

        Ok(())
    }

    /// # Errors
    /// - if the database connection cannot be established
    /// - if the database query fails
    /// - If the entity is not found
    async fn get_entity(&self, id: Uuid, user_id: Uuid) -> Result<Entity, DBError> {
        let client = self.pool.get().await?;

        let stmt = client
            .prepare("SELECT id FROM entities WHERE id = $1 and user_id = $2")
            .await?;
        let row = client.query_opt(&stmt, &[&id, &user_id]).await?;
        let Some(row) = row else {
            return Err(DBError::NotFound);
        };

        Ok(Entity::try_from(row)?)
    }
}

impl TryFrom<Row> for Entity {
    type Error = DBError;

    fn try_from(value: Row) -> Result<Self, DBError> {
        let id: Uuid = value.try_get("id")?;

        Ok(Entity { id: id.to_string() })
    }
}

#[cfg(test)]
pub mod test {
    use crate::SERVICE_NAME;
    use crate::{
        proto::Entity,
        utils::test::{fixture_entity, fixture_uuid},
    };
    use shared::test_utils::get_test_db;
    use tokio::sync::Mutex;
    use tonic::async_trait;
    use uuid::Uuid;

    use super::*;

    use crate::error::DBError;

    pub struct MockDBClient {
        pub get_entity: Mutex<Option<Result<Entity, DBError>>>,
        pub insert_entity: Mutex<Option<Result<(), DBError>>>,
    }

    impl Default for MockDBClient {
        fn default() -> Self {
            Self {
                insert_entity: Mutex::new(None),
                get_entity: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl DBClient for MockDBClient {
        async fn insert_entity(&self, _: Uuid, _: Uuid) -> Result<(), DBError> {
            self.insert_entity.lock().await.take().unwrap()
        }

        async fn get_entity(&self, _: Uuid, _: Uuid) -> Result<Entity, DBError> {
            self.get_entity.lock().await.take().unwrap()
        }
    }

    #[derive(Clone)]
    struct DBEntity {
        id: Uuid,
        user_id: Uuid,
    }

    fn fixture_db_entity<F>(mut func: F) -> DBEntity
    where
        F: FnMut(&mut DBEntity),
    {
        let mut entity = DBEntity {
            id: fixture_uuid(),
            user_id: fixture_uuid(),
        };
        func(&mut entity);
        entity
    }

    async fn run_db_test<F, Fut>(given_entity: Vec<DBEntity>, test_fn: F)
    where
        F: FnOnce(PostgresDBClient) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let migrations = std::fs::canonicalize("./migrations").unwrap();
        let pool = get_test_db(SERVICE_NAME, migrations)
            .await
            .expect("failed to get connection to test db");
        let db_client = PostgresDBClient { pool };

        for entity in given_entity {
            db_client
                .insert_entity(entity.id.clone(), entity.user_id.clone())
                .await
                .expect("failed to insert entity");
        }

        test_fn(db_client).await;
    }

    #[tokio::test]
    async fn test_get_entity() {
        let id = fixture_uuid();
        let user_id = fixture_uuid();
        let given_entity = fixture_db_entity(|u| u.id = id);
        let want_entity = fixture_entity(|u| u.id = id.to_string());

        run_db_test(vec![given_entity], |db_client| async move {
            let got_entity = db_client
                .get_entity(id, user_id)
                .await
                .expect("failed to get entity");

            assert_eq!(got_entity, want_entity);
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_entity_not_found() {
        let id = Uuid::parse_str("99999999-9999-9999-9999-999999999999").unwrap();
        let user_id = fixture_uuid();

        run_db_test(vec![], |db_client| async move {
            let got_result = db_client.get_entity(id, user_id).await;

            assert!(matches!(got_result, Err(DBError::NotFound)));
        })
        .await;
    }
}
