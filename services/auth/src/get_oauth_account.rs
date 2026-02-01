use crate::{
    db::DBClient,
    error::Error,
    handler::Handler,
    proto::{GetOauthAccountReq, GetOauthAccountResp},
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
    pub async fn get_oauth_account(
        &self,
        req: Request<GetOauthAccountReq>,
    ) -> Result<Response<GetOauthAccountResp>, Status> {
        let req = req.into_inner();

        let user_id = validate_user_id(&req.user_id)?;

        let account = self
            .db
            .get_oauth_account(user_id, req.provider())
            .await
            .map_err(Error::GetOauthAccount)?;

        Ok(Response::new(GetOauthAccountResp {
            external_user_id: account.external_user_id,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::test::MockDBClient,
        error::DBError,
        handler::Handler,
        oauth::{github::GithubOAuth, google::GoogleOAuth},
        proto::{GetOauthAccountReq, GetOauthAccountResp, OauthProvider},
        utils::{OAuthAccount, tests::fixture_oauth_account},
    };
    use common::mock::MockNow;
    use oauth::mock::MockRandom;
    use rstest::rstest;
    use std::marker::PhantomData;
    use testutils::assert_response;
    use tokio::sync::Mutex;
    use tonic::{Code, Request};

    fn fixture_uuid() -> uuid::Uuid {
        uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap()
    }

    #[rstest]
    #[case::happy_path(
        GetOauthAccountReq {
            user_id: fixture_uuid().to_string(),
            provider: OauthProvider::Google as i32,
        },
        Ok(fixture_oauth_account(|_| {})),
        Ok(GetOauthAccountResp {
            external_user_id: "external-user-id".to_string(),
        })
    )]
    #[case::missing_user_id(
        GetOauthAccountReq {
            user_id: String::new(),
            ..Default::default()
        },
        Ok(fixture_oauth_account(|_| {})),
        Err(Code::InvalidArgument)
    )]
    #[tokio::test]
    async fn test_get_oauth_account(
        #[case] req: GetOauthAccountReq,
        #[case] db_result: Result<OAuthAccount, DBError>,
        #[case] want: Result<GetOauthAccountResp, Code>,
    ) {
        // given
        let db = MockDBClient {
            get_oauth_account: Mutex::new(Some(db_result)),
            ..Default::default()
        };
        let handler = Handler {
            db,
            google: GoogleOAuth::<MockRandom>::default(),
            github: GithubOAuth::<MockRandom>::default(),
            _now: PhantomData::<MockNow>,
        };

        // when
        let got = handler.get_oauth_account(Request::new(req)).await;

        // then
        assert_response(got, want);
    }
}
