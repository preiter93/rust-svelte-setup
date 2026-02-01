use tonic::{Request, Response, Status};

use crate::{
    db::DBClient,
    error::Error,
    handler::Handler,
    proto::{DeleteSessionReq, DeleteSessionResp},
};

impl<D, R, N> Handler<D, R, N>
where
    D: DBClient,
{
    /// Deletes a session.
    ///
    /// # Errors
    /// - token is malformed
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    pub async fn delete_session(
        &self,
        req: Request<DeleteSessionReq>,
    ) -> Result<Response<DeleteSessionResp>, Status> {
        let token = req.into_inner().token;

        if token.is_empty() {
            return Err(Error::MissingToken.into());
        }

        let token_parts: Vec<_> = token.split('.').collect();
        if token_parts.len() != 2 {
            return Err(Error::InvalidToken.into());
        }

        let session_id = token_parts[0];

        self.db
            .delete_session(session_id)
            .await
            .map_err(Error::DeleteSession)?;

        Ok(Response::new(DeleteSessionResp {}))
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use common::mock::MockNow;
    use oauth::mock::MockRandom;
    use rstest::rstest;
    use testutils::assert_response;
    use tokio::sync::Mutex;
    use tonic::{Code, Request};

    use crate::{
        db::test::MockDBClient,
        error::DBError,
        handler::Handler,
        oauth::{github::GithubOAuth, google::GoogleOAuth},
        proto::{DeleteSessionReq, DeleteSessionResp},
        utils::tests::fixture_token,
    };

    #[rstest]
    #[case::happy_path(
        DeleteSessionReq {
            token: fixture_token(),
        },
        Ok(()),
        Ok(DeleteSessionResp {})
    )]
    #[case::missing_token(
        DeleteSessionReq {
            token: String::new(),
        },
        Ok(()),
        Err(Code::InvalidArgument)
    )]
    #[case::invalid_format(
        DeleteSessionReq {
            token: "invalid-format".to_string(),
        },
        Ok(()),
        Err(Code::InvalidArgument)
    )]
    #[case::db_error(
        DeleteSessionReq {
            token: fixture_token(),
        },
        Err(DBError::Unknown),
        Err(Code::Internal)
    )]
    #[tokio::test]
    async fn test_delete_session(
        #[case] req: DeleteSessionReq,
        #[case] db_result: Result<(), DBError>,
        #[case] want: Result<DeleteSessionResp, Code>,
    ) {
        // given
        let db = MockDBClient {
            delete_session: Mutex::new(Some(db_result)),
            ..Default::default()
        };
        let handler = Handler {
            db,
            google: GoogleOAuth::<MockRandom>::default(),
            github: GithubOAuth::<MockRandom>::default(),
            _now: PhantomData::<MockNow>,
        };

        // when
        let got = handler.delete_session(Request::new(req)).await;

        // then
        assert_response(got, want);
    }
}
