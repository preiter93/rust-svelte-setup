/// The session token cookie key.
pub const SESSION_TOKEN_COOKIE_KEY: &'static str = "session_token";

/// The session token expiry time in seconds.
pub const SESSION_TOKEN_EXPIRY_TIME: i64 = 7 * 60 * 60 * 24; // 7 days

/// Represents session state.
#[derive(Clone, Debug)]
pub struct SessionState {
    /// The id of the authenticated used.
    pub user_id: String,
}

impl SessionState {
    /// Creates a new `SessionState`.
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }
}
