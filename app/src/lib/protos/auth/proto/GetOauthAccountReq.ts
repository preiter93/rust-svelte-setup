// Original file: /Users/philippreiter/Svelte/rust-svelte-setup/services/auth/api.proto

import type { OauthProvider as _proto_OauthProvider, OauthProvider__Output as _proto_OauthProvider__Output } from '../proto/OauthProvider';

export interface GetOauthAccountReq {
  'user_id'?: (string);
  'provider'?: (_proto_OauthProvider);
}

export interface GetOauthAccountReq__Output {
  'user_id': (string);
  'provider': (_proto_OauthProvider__Output);
}
