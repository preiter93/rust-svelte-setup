use crate::error::{DBError, Error};
use crate::utils::validate_entity_id;

use crate::{
    db::DBClient,
    proto::{GetEntityReq, GetEntityResp},
    server::Server,
};
use common::UuidGenerator;
use setup::validate_user_id;
use tonic::{Request, Response, Status};

impl<D, U> Server<D, U>
where
    D: DBClient,
    U: UuidGenerator,
{
    /// Gets an entity by identifier.
    ///
    /// # Errors
    /// - ?
    pub async fn get_entity(
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
    use rstest::rstest;
    use tokio::sync::Mutex;
    use tonic::{Code, Request};

    use crate::{
        db::test::MockDBClient,
        error::DBError,
        proto::{Entity, GetEntityReq, GetEntityResp},
        server::Server,
        utils::test::{fixture_entity, fixture_get_entity_req, fixture_get_entity_resp},
    };

    #[rstest]
    #[case::happy_path(
        fixture_get_entity_req(|_| {}),
        Ok(fixture_entity(|_| {})),
        Ok(fixture_get_entity_resp(|_| {}))
    )]
    #[case::missing_id(
        fixture_get_entity_req(|v| { v.id = String::new(); }),
        Ok(fixture_entity(|_| {})),
        Err(Code::InvalidArgument)
    )]
    #[case::missing_user_id(
        fixture_get_entity_req(|v| { v.user_id = String::new(); }),
        Ok(fixture_entity(|_| {})),
        Err(Code::InvalidArgument)
    )]
    #[case::not_found(
        fixture_get_entity_req(|_| {}),
        Err(DBError::NotFound),
        Err(Code::NotFound)
    )]
    #[case::internal_error(
        fixture_get_entity_req(|_| {}),
        Err(DBError::Unknown),
        Err(Code::Internal)
    )]
    #[tokio::test]
    async fn test_get_entity(
        #[case] req: GetEntityReq,
        #[case] db_result: Result<Entity, DBError>,
        #[case] want: Result<GetEntityResp, Code>,
    ) {
        // given
        use common::mock::MockUuidGenerator;
        use testutils::assert_response;
        let db = MockDBClient {
            get_entity: Mutex::new(Some(db_result)),
            ..Default::default()
        };
        let service = Server {
            db,
            uuid: MockUuidGenerator::default(),
        };

        // when
        let got = service.get_entity(Request::new(req)).await;

        // then
        assert_response(got, want);
    }
}
