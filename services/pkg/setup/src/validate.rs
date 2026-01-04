//! Convenient helper methods to deal with user validation.
use std::str::FromStr;
use tonic::{Code, Status};
use uuid::Uuid;

pub fn validate_user_id(user_id: &str) -> Result<Uuid, ValidateUserError> {
    if user_id.is_empty() {
        return Err(ValidateUserError::MissingUserId);
    }

    let Ok(user_uuid) = Uuid::from_str(user_id) else {
        return Err(ValidateUserError::InvalidUserId(user_id.to_string()));
    };

    tracing::Span::current().record("user_id", user_id);

    Ok(user_uuid)
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateUserError {
    #[error("missing user id")]
    MissingUserId,
    #[error("invalid user id: {0}")]
    InvalidUserId(String),
}

impl From<ValidateUserError> for Status {
    fn from(err: ValidateUserError) -> Self {
        let code = match err {
            ValidateUserError::MissingUserId | ValidateUserError::InvalidUserId(_) => {
                Code::InvalidArgument
            }
        };
        Status::new(code, err.to_string())
    }
}
