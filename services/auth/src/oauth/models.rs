use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OAuth2Token {
    pub access_token: Option<String>,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
    pub token_type: Option<String>,
    pub id_token: Option<String>,
}
