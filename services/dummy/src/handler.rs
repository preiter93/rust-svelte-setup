use crate::error::{DBError, Error};
use crate::utils::{UuidGenerator, validate_entity_id};

use crate::{
    db::DBClient,
    proto::{GetEntityReq, GetEntityResp, api_service_server::ApiService},
};
use shared::helper::validate_user_id;
use tonic::{Request, Response, Status};
use tracing::instrument;

#[derive(Clone)]
pub(crate) struct Handler<D, U> {
    pub db: D,
    #[allow(dead_code)]
    pub uuid: U,
}

#[tonic::async_trait]
impl<D, U> ApiService for Handler<D, U>
where
    D: DBClient,
    U: UuidGenerator,
{
    /// Gets an entity by identifier.
    ///
    /// # Errors
    /// - ?
    #[instrument(skip_all, fields(user_id), err)]
    async fn get_entity(
        &self,
        req: Request<GetEntityReq>,
    ) -> Result<Response<GetEntityResp>, Status> {
        let req = req.into_inner();

        let user_id = validate_user_id(&req.user_id)?;

        let id = validate_entity_id(&req.id)?;

        let entity = self.db.get_entity(id, user_id).await.map_err(|e| match e {
            DBError::NotFound => Error::EntityNotFound(id.to_string()),
            _ => Error::GetEntity(e),
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
        proto::{Entity, GetEntityReq, GetEntityResp, api_service_server::ApiService as _},
        utils::test::{
            MockUuidGenerator, assert_response, fixture_entity, fixture_get_entity_req,
            fixture_get_entity_resp,
        },
    };

    struct TestCaseGetEntity {
        given_req: GetEntityReq,
        given_db_get_entity: Result<Entity, DBError>,
        want_resp: Result<GetEntityResp, Code>,
    }

    impl Default for TestCaseGetEntity {
        fn default() -> Self {
            Self {
                given_req: fixture_get_entity_req(|_| {}),
                given_db_get_entity: Ok(fixture_entity(|_| {})),
                want_resp: Ok(fixture_get_entity_resp(|_| {})),
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
            let req = Request::new(self.given_req);
            let got = service.get_entity(req).await;

            // then
            assert_response(got, self.want_resp);
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
            given_req: fixture_get_entity_req(|v| {
                v.id = String::new();
            }),
            want_resp: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_entity_missing_user_id() {
        TestCaseGetEntity {
            given_req: fixture_get_entity_req(|v| {
                v.user_id = String::new();
            }),
            want_resp: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_entity_not_found() {
        TestCaseGetEntity {
            given_db_get_entity: Err(DBError::NotFound),
            want_resp: Err(Code::NotFound),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_entity_internal_error() {
        TestCaseGetEntity {
            given_db_get_entity: Err(DBError::Unknown),
            want_resp: Err(Code::Internal),
            ..Default::default()
        }
        .run()
        .await;
    }
}
