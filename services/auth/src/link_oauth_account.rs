use crate::{
    db::DBClient,
    error::Error,
    handler::Handler,
    proto::{LinkOauthAccountReq, LinkOauthAccountResp},
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
    /// Links a user_id to an oauth token.
    ///
    /// # Errors
    /// - missing oauth token id
    /// - missing user id
    /// - updating oauth token (db)
    pub async fn link_oauth_account(
        &self,
        req: Request<LinkOauthAccountReq>,
    ) -> Result<Response<LinkOauthAccountResp>, Status> {
        let req = req.into_inner();

        let account_id = req.account_id;
        if account_id.is_empty() {
            return Err(Error::MissingOauthAccountID.into());
        }

        let user_id = validate_user_id(&req.user_id)?;

        self.db
            .update_oauth_account(&account_id, user_id)
            .await
            .map_err(Error::UpdateOauthAccount)?;

        Ok(Response::new(LinkOauthAccountResp {}))
    }
}
