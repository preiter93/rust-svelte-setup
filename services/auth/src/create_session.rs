use crate::{
    db::DBClient,
    error::Error,
    handler::{Handler, SessionToken},
    proto::{CreateSessionReq, CreateSessionResp},
    utils::{Session, hash_secret},
};
use common::Now;
use oauth::RandomSource;
use setup::validate_user_id;
use tonic::{Request, Response, Status};

impl<D, R, N> Handler<D, R, N>
where
    D: DBClient,
    R: RandomSource + Clone,
    N: Now,
{
    /// Creates a new session.
    ///
    /// # Errors
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    pub async fn create_session(
        &self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status> {
        let req = req.into_inner();

        let user_id = validate_user_id(&req.user_id)?;

        let id = R::alphanumeric(24);
        let secret = R::alphanumeric(24);
        let token: SessionToken = format!("{id}.{secret}");

        let session = Session {
            id,
            secret_hash: hash_secret(&secret),
            created_at: N::now(),
            user_id,
            ..Default::default()
        };

        self.db
            .insert_session(session)
            .await
            .map_err(Error::InsertSession)?;

        Ok(Response::new(CreateSessionResp { token }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test::MockDBClient;
    use crate::error::DBError;
    use crate::oauth::{github::GithubOAuth, google::GoogleOAuth};
    use crate::utils::tests::{fixture_token, fixture_uuid};
    use common::mock::MockNow;
    use oauth::mock::MockRandom;
    use rstest::rstest;
    use std::marker::PhantomData;
    use testutils::assert_response;
    use tokio::sync::Mutex;
    use tonic::Code;

    #[rstest]
    #[case::happy_path(
        CreateSessionReq {
            user_id: fixture_uuid().to_string(),
        },
        Ok(()),
        Ok(CreateSessionResp {
            token: fixture_token(),
        })
    )]
    #[case::missing_user_id(
        CreateSessionReq {
            user_id: String::new(),
        },
        Ok(()),
        Err(Code::InvalidArgument)
    )]
    #[case::db_error(
        CreateSessionReq {
            user_id: fixture_uuid().to_string(),
        },
        Err(DBError::Unknown),
        Err(Code::Internal)
    )]
    #[tokio::test]
    async fn test_create_session(
        #[case] req: CreateSessionReq,
        #[case] db_result: Result<(), DBError>,
        #[case] want: Result<CreateSessionResp, Code>,
    ) {
        // given
        let db = MockDBClient {
            insert_session: Mutex::new(Some(db_result)),
            ..Default::default()
        };
        let handler = Handler {
            db,
            google: GoogleOAuth::<MockRandom>::default(),
            github: GithubOAuth::<MockRandom>::default(),
            _now: PhantomData::<MockNow>,
        };

        // when
        let got = handler.create_session(Request::new(req)).await;

        // then
        assert_response(got, want);
    }
}
