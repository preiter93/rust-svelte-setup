CREATE TABLE IF NOT EXISTS oauth_accounts (
  id                      TEXT        NOT NULL PRIMARY KEY,
  provider                INTEGER     NOT NULL,
  provider_user_id        TEXT        NOT NULL UNIQUE,
  provider_user_name      TEXT        NULL,
  provider_user_email     TEXT        NULL,
  access_token            TEXT        NULL,
  access_token_expires_at TIMESTAMPTZ NULL,
  refresh_token           TEXT        NULL,
  user_id                 TEXT        NULL,
  created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
