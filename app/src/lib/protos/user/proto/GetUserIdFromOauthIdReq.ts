// Original file: /Users/philippreiter/Svelte/rust-svelte-setup/services/user/api.proto

import type { OauthProvider as _proto_OauthProvider, OauthProvider__Output as _proto_OauthProvider__Output } from '../proto/OauthProvider';

export interface GetUserIdFromOauthIdReq {
  'oauth_id'?: (string);
  'provider'?: (_proto_OauthProvider);
}

export interface GetUserIdFromOauthIdReq__Output {
  'oauth_id': (string);
  'provider': (_proto_OauthProvider__Output);
}
