use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;

use crate::Server;

#[debug_handler]
pub async fn get_session(State(_): State<Server>) -> Result<Json<String>, GetSessionError> {
    todo!()
}

pub struct GetSessionError;

impl IntoResponse for GetSessionError {
    fn into_response(self) -> Response {
        todo!()
    }
}
