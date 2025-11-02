CREATE TABLE IF NOT EXISTS oauth_accounts (
  id                      TEXT        NOT NULL PRIMARY KEY,
  provider                INTEGER     NOT NULL,
  external_user_id        TEXT        NOT NULL UNIQUE,
  external_user_name      TEXT        NULL,
  external_user_email     TEXT        NULL,
  access_token            TEXT        NULL,
  access_token_expires_at TIMESTAMPTZ NULL,
  refresh_token           TEXT        NULL,
  user_id                 UUID        NULL,
  created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
