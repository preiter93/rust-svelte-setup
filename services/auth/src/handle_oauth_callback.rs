use tonic::{Request, Response, Status};

use crate::{
    db::DBClient,
    error::Error,
    handler::Handler,
    proto::{HandleOauthCallbackReq, HandleOauthCallbackResp, OauthProvider},
};
use common::Now;
use oauth::{OAuthProvider as _, RandomSource};

impl<D, R, N> Handler<D, R, N>
where
    D: DBClient,
    R: RandomSource + Clone,
    N: Now,
{
    /// Handles a oauth login callback
    ///
    /// # Errors
    /// - validating authorization code
    /// - decoding the id token
    /// - upserting oauth token (db)
    pub async fn handle_oauth_callback(
        &self,
        req: Request<HandleOauthCallbackReq>,
    ) -> Result<Response<HandleOauthCallbackResp>, Status> {
        let req = req.into_inner();

        let (code, code_verifier) = (&req.code, &req.code_verifier);

        let account = match req.provider() {
            OauthProvider::Google => self.google.exchange_code(code, code_verifier).await,
            OauthProvider::Github => self.github.exchange_code(code, code_verifier).await,
            _ => return Err(Error::UnspecifiedOauthProvider.into()),
        }?;

        let account = self
            .db
            .upsert_oauth_account(&account)
            .await
            .map_err(Error::UpsertOauthAccount)?;

        Ok(Response::new(HandleOauthCallbackResp {
            account_id: account.id,
            external_user_name: account.external_user_name.unwrap_or_default(),
            external_user_email: account.external_user_email.unwrap_or_default(),
            user_id: account.user_id.map(|e| e.to_string()).unwrap_or_default(),
        }))
    }
}
