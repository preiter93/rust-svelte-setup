use oauth::{OAuth, OAuthProvider as _, RandomSource};
use tonic::{Request, Response, Status};

use crate::{
    error::Error,
    proto::{OauthProvider, StartOauthLoginReq, StartOauthLoginResp},
    server::Server,
};

impl<D, R, N> Server<D, R, N>
where
    R: RandomSource + Clone,
{
    /// Starts a oauth login.
    ///
    /// # Errors
    /// - generating authorization url
    pub async fn start_oauth_login(
        &self,
        req: Request<StartOauthLoginReq>,
    ) -> Result<Response<StartOauthLoginResp>, Status> {
        let req = req.into_inner();

        let state = OAuth::<R>::generate_state();
        let (code_verifier, authorization_url) = match req.provider() {
            OauthProvider::Google => {
                let verifier = OAuth::<R>::generate_code_verifier();
                let challenge = OAuth::<R>::create_s256_code_challenge(&verifier);

                let auth_url = self.google.generate_authorization_url(&state, &challenge)?;

                (verifier, auth_url)
            }
            OauthProvider::Github => {
                let auth_url = self.github.generate_authorization_url(&state, "")?;

                (String::new(), auth_url)
            }
            _ => return Err(Error::UnspecifiedOauthProvider.into()),
        };

        Ok(Response::new(StartOauthLoginResp {
            state,
            code_verifier,
            authorization_url,
        }))
    }
}
