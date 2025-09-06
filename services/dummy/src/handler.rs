use std::str::FromStr;

use crate::error::{CreateEntityErr, DBError, GetEntityErr};
use crate::utils::UuidGenerator;

use crate::{
    db::DBClient,
    proto::{
        CreateEntityReq, CreateEntityResp, GetEntityReq, GetEntityResp,
        api_service_server::ApiService,
    },
};
use tonic::{Request, Response, Status};
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct Handler<D, U> {
    pub db: D,
    pub uuid: U,
}

#[tonic::async_trait]
impl<D, U> ApiService for Handler<D, U>
where
    D: DBClient,
    U: UuidGenerator,
{
    /// Creates a new entity.
    ///
    /// # Errors
    /// - ?
    #[instrument(skip_all, err)]
    async fn create_entity(
        &self,
        _: Request<CreateEntityReq>,
    ) -> Result<Response<CreateEntityResp>, Status> {
        let id = self.uuid.generate();

        self.db
            .insert_entity(id)
            .await
            .map_err(CreateEntityErr::Database)?;

        let resp = CreateEntityResp { id: id.to_string() };

        Ok(Response::new(resp))
    }

    /// Gets an entity by identifier.
    ///
    /// # Errors
    /// - ?
    #[instrument(skip_all, err)]
    async fn get_entity(
        &self,
        req: Request<GetEntityReq>,
    ) -> Result<Response<GetEntityResp>, Status> {
        let req = req.into_inner();
        if req.id.is_empty() {
            return Err(GetEntityErr::MissingEntityId.into());
        }

        let id = Uuid::from_str(&req.id).map_err(|_| GetEntityErr::NotAUUID)?;

        let entity = self.db.get_entity(id).await.map_err(|e| match e {
            DBError::NotFound => GetEntityErr::NotFound(req.id),
            _ => GetEntityErr::Database(e),
        })?;

        Ok(Response::new(GetEntityResp {
            entity: Some(entity),
        }))
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;
    use tonic::{Code, Request};

    use crate::{
        db::test::MockDBClient,
        error::DBError,
        handler::Handler,
        proto::{
            CreateEntityReq, CreateEntityResp, Entity, GetEntityReq, GetEntityResp,
            api_service_server::ApiService as _,
        },
        utils::test::{
            MockUuidGenerator, assert_response, fixture_create_entity_req, fixture_entity,
            fixture_uuid, fixture_uuid_string,
        },
    };

    struct TestCaseCreateEntity {
        given_req: CreateEntityReq,
        given_db_insert_entity: Result<(), DBError>,
        want: Result<CreateEntityResp, Code>,
    }

    impl Default for TestCaseCreateEntity {
        fn default() -> Self {
            Self {
                given_req: fixture_create_entity_req(|_| {}),
                given_db_insert_entity: Ok(()),
                want: Ok(CreateEntityResp {
                    id: fixture_uuid_string(),
                }),
            }
        }
    }

    impl TestCaseCreateEntity {
        async fn run(self) {
            // given
            let db = MockDBClient {
                insert_entity: Mutex::new(Some(self.given_db_insert_entity)),
                ..Default::default()
            };
            let uuid = MockUuidGenerator::default();
            let service = Handler { db, uuid };

            // when
            let req = Request::new(self.given_req);
            let got = service.create_entity(req).await;

            // then
            assert_response(got, self.want);
        }
    }

    #[tokio::test]
    async fn test_create_entity_happy_path() {
        TestCaseCreateEntity {
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_create_entity_internal_error() {
        TestCaseCreateEntity {
            given_db_insert_entity: Err(DBError::Unknown),
            want: Err(Code::Internal),
            ..Default::default()
        }
        .run()
        .await;
    }

    struct TestCaseGetEntity {
        given_id: String,
        given_db_get_entity: Result<Entity, DBError>,
        want: Result<GetEntityResp, Code>,
    }

    impl Default for TestCaseGetEntity {
        fn default() -> Self {
            Self {
                given_id: fixture_uuid().to_string(),
                given_db_get_entity: Ok(fixture_entity(|_| {})),
                want: Ok(GetEntityResp {
                    entity: Some(fixture_entity(|_| {})),
                }),
            }
        }
    }

    impl TestCaseGetEntity {
        async fn run(self) {
            // given
            let db = MockDBClient {
                get_entity: Mutex::new(Some(self.given_db_get_entity)),
                ..Default::default()
            };
            let uuid = MockUuidGenerator::default();
            let service = Handler { db, uuid };

            // when
            let req = Request::new(GetEntityReq { id: self.given_id });
            let got = service.get_entity(req).await;

            // then
            assert_response(got, self.want);
        }
    }

    #[tokio::test]
    async fn test_get_entity_happy_path() {
        TestCaseGetEntity {
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_entity_missing_id() {
        TestCaseGetEntity {
            given_id: String::new(),
            want: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_entity_not_found() {
        TestCaseGetEntity {
            given_db_get_entity: Err(DBError::NotFound),
            want: Err(Code::NotFound),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_entity_internal_error() {
        TestCaseGetEntity {
            given_db_get_entity: Err(DBError::Unknown),
            want: Err(Code::Internal),
            ..Default::default()
        }
        .run()
        .await;
    }
}
