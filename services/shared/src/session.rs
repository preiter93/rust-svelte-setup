/// Holds session state.
#[derive(Clone, Debug)]
pub struct SessionState {
    /// The user id.
    pub user_id: String,
}

impl SessionState {
    /// Creates a new `SessionState`.
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }
}
