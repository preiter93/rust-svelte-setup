//! # Session-based authentication:
//! - The user logs in with a username and password
//! - The server authenticates the user and generates a session token
//! - The session token is stored in the database together with user info
//! - The token is sent to the client and stored in a cookie or local storage
//! - For requests the client sends the session token
//! - The server fetches user id from the token via the database and authorizes the user
//!
//! # Further readings
//! <https://lucia-auth.com/sessions/basic>
use chrono::{DateTime, Duration, Utc};
use tonic::{Request, Response, Status};
use tracing::instrument;

use crate::{
    db::DBCLient,
    error::{
        CreateSessionErr, DBError, HandleGoogleCallbackErr, StartGoogleLoginErr, ValidateSessionErr,
    },
    proto::{
        CreateSessionReq, CreateSessionResp, DeleteSessionReq, DeleteSessionResp,
        HandleGoogleCallbackReq, HandleGoogleCallbackResp, StartGoogleLoginReq,
        StartGoogleLoginResp, ValidateSessionReq, ValidateSessionResp,
        api_service_server::ApiService,
    },
    utils::{GoogleOAuth, OAuth, constant_time_equal, generate_secure_random_string, hash_secret},
};

const SESSION_EXPIRES_IN_SECONDS: i64 = 60 * 60 * 24; // 1 day

#[derive(Clone)]
pub struct Handler {
    pub db: DBCLient,
    pub google: GoogleOAuth,
}

type SessionToken = String;

#[tonic::async_trait]
impl ApiService for Handler {
    /// Creates a new session.
    ///
    /// # Errors
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    #[instrument(skip(self), err)]
    async fn create_session(
        &self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status> {
        let req = req.into_inner();
        if req.user_id.is_empty() {
            return Err(CreateSessionErr::MissingUserUID.into());
        }

        let now: DateTime<Utc> = Utc::now();

        let id = generate_secure_random_string();
        let secret = generate_secure_random_string();
        let secret_hash = hash_secret(&secret);

        self.db
            .insert_session(&id, &secret_hash, &req.user_id, now)
            .await
            .map_err(CreateSessionErr::Database)?;

        Ok(Response::new(CreateSessionResp {
            token: format!("{id}.{secret}"),
        }))
    }

    /// Validates a sessions token by parsing out the id and secret
    /// from the token, getting the session with the id, checking
    /// the expiration and comparing the secret against the hash.
    ///
    /// # Errors
    /// - token is malformed
    /// - session is expired
    /// - session secret is invalid
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    #[instrument(skip(self), err)]
    async fn validate_session(
        &self,
        req: Request<ValidateSessionReq>,
    ) -> Result<Response<ValidateSessionResp>, Status> {
        let token = req.into_inner().token;
        let token_parts: Vec<_> = token.split('.').collect();
        if token_parts.len() != 2 {
            return Err(ValidateSessionErr::InvalidFormat.into());
        }

        let session_id = token_parts[0];
        let session_secret = token_parts[1];

        let session = self.db.get_session(session_id).await.map_err(|e| match e {
            DBError::NotFound => ValidateSessionErr::NotFound,
            _ => ValidateSessionErr::Database(e),
        })?;

        let is_expired = Utc::now().signed_duration_since(session.created_at)
            >= Duration::seconds(SESSION_EXPIRES_IN_SECONDS);
        if is_expired {
            self.db
                .delete_session(&session.id)
                .await
                .map_err(ValidateSessionErr::Database)?;
            return Err(ValidateSessionErr::Expired.into());
        }

        let token_secret_hash = hash_secret(session_secret);
        let valid_secret = constant_time_equal(&token_secret_hash, &session.secret_hash);
        if !valid_secret {
            return Err(ValidateSessionErr::SecretMismatch.into());
        }

        Ok(Response::new(ValidateSessionResp {
            user_id: session.user_id,
        }))
    }

    /// Deletes a session.
    ///
    /// # Errors
    /// - token is malformed
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    #[instrument(skip(self), err)]
    async fn delete_session(
        &self,
        req: Request<DeleteSessionReq>,
    ) -> Result<Response<DeleteSessionResp>, Status> {
        let token = req.into_inner().token;
        let token_parts: Vec<_> = token.split('.').collect();
        if token_parts.len() != 2 {
            return Err(ValidateSessionErr::InvalidFormat.into());
        }

        let session_id = token_parts[0];

        self.db
            .delete_session(session_id)
            .await
            .map_err(CreateSessionErr::Database)?;

        Ok(Response::new(DeleteSessionResp {}))
    }

    /// Starts a google login.
    ///
    /// # Errors
    /// - generating authorization url
    #[instrument(skip(self), err)]
    async fn start_google_login(
        &self,
        _: Request<StartGoogleLoginReq>,
    ) -> Result<Response<StartGoogleLoginResp>, Status> {
        let (state, code_verifier) = (OAuth::generate_state(), OAuth::generate_code_verifier());

        let authorization_url = self
            .google
            .generate_authorization_url(&state, &code_verifier)
            .map_err(|_| StartGoogleLoginErr::AuthorizationUrl)?;

        Ok(Response::new(StartGoogleLoginResp {
            state,
            code_verifier,
            authorization_url,
        }))
    }

    /// Handles a google login callback
    ///
    /// # Errors
    /// - validating authorization code
    /// - decoding the id token
    async fn handle_google_callback(
        &self,
        req: Request<HandleGoogleCallbackReq>,
    ) -> Result<Response<HandleGoogleCallbackResp>, Status> {
        let req = req.into_inner();
        let tokens = self
            .google
            .validate_authorization_code(&req.code, &req.code_verifier)
            .await
            .map_err(|_| HandleGoogleCallbackErr::ValidateAuthorizationCode)?;

        let claims = self
            .google
            .decode_id_token(&tokens.id_token)
            .await
            .map_err(|_| HandleGoogleCallbackErr::DecodeIdToken)?;

        return Ok(Response::new(HandleGoogleCallbackResp {
            google_id: claims.sub,
            name: claims.name,
            email: claims.email,
        }));
    }
}
