// Original file: /Users/philippreiter/Svelte/rust-svelte-setup/services/auth/api.proto

import type { OauthProvider as _proto_OauthProvider, OauthProvider__Output as _proto_OauthProvider__Output } from '../proto/OauthProvider';

export interface HandleOauthCallbackReq {
  'provider'?: (_proto_OauthProvider);
  'code'?: (string);
  'code_verifier'?: (string);
}

export interface HandleOauthCallbackReq__Output {
  'provider': (_proto_OauthProvider__Output);
  'code': (string);
  'code_verifier': (string);
}
