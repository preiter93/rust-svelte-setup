use serde::Deserialize;

/// Standard OpenID Connect (OIDC) ID token claims.
///
/// These fields are defined in the OIDC Core specification, but not all providers
/// include all of them.
#[derive(Debug, Clone, Deserialize)]
pub struct OidcTokenClaims {
    /// Subject Identifier — unique and stable per user.
    pub sub: String,

    /// Issuer Identifier — identifies the authorization server.
    pub iss: Option<String>,

    /// Audience(s) — identifies the recipients for which the token is intended.
    pub aud: Option<Vec<String>>,

    /// Expiration time (UNIX timestamp).
    pub exp: Option<u64>,

    /// Issued-at time (UNIX timestamp).
    pub iat: Option<u64>,

    /// Authorized party (client_id of the relying party).
    pub azp: Option<String>,

    /// The user's email address.
    pub email: Option<String>,

    /// Whether the email address is verified.
    pub email_verified: Option<bool>,

    /// The user's display name.
    pub name: Option<String>,

    /// Given name (first name).
    pub given_name: Option<String>,

    /// Family name (last name).
    pub family_name: Option<String>,

    /// Preferred username or handle.
    pub preferred_username: Option<String>,

    /// Locale (e.g., "en-US").
    pub locale: Option<String>,

    /// Profile picture URL.
    pub picture: Option<String>,
}

/// Represents a JSON Web Key Set (JWKS).
#[derive(Debug, Deserialize)]
pub(crate) struct Jwks {
    /// The list of JSON Web Keys.
    pub(crate) keys: Vec<Jwk>,
}

/// Represents a single JSON Web Key (JWK).
#[derive(Debug, Deserialize)]
pub(crate) struct Jwk {
    /// Key ID
    pub(crate) kid: String,
    /// RSA modulus
    pub(crate) n: String,
    /// RSA exponent
    pub(crate) e: String,
}
