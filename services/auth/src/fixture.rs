#![cfg(test)]

use chrono::TimeZone;
use uuid::Uuid;

use crate::utils::{DBSession, OAuthAccount, hash_secret};

pub fn fixture_uuid() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
}

pub(crate) fn fixture_token() -> String {
    "secret.secret".to_string()
}

pub(crate) fn fixture_db_session<F>(mut func: F) -> DBSession
where
    F: FnMut(&mut DBSession),
{
    let mut session = DBSession {
        id: "session-id".to_string(),
        secret_hash: hash_secret("secret"),
        created_at: chrono::Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        expires_at: chrono::Utc.with_ymd_and_hms(2020, 1, 8, 0, 0, 0).unwrap(),
        user_id: fixture_uuid(),
    };
    func(&mut session);
    session
}

pub(crate) fn fixture_oauth_account<F>(mut func: F) -> OAuthAccount
where
    F: FnMut(&mut OAuthAccount),
{
    let mut token = OAuthAccount {
        id: "oauth-id".to_string(),
        external_user_id: "external-user-id".to_string(),
        external_user_name: Some("external-user-name".to_string()),
        external_user_email: Some("external-user-email".to_string()),
        provider: 0,
        access_token: Some("access-token".to_string()),
        access_token_expires_at: None,
        refresh_token: None,
        user_id: None,
    };
    func(&mut token);
    token
}
