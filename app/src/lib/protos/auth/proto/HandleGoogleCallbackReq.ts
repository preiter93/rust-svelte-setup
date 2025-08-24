// Original file: /Users/philippreiter/Svelte/rust-svelte-setup/services/auth/api.proto


export interface HandleGoogleCallbackReq {
  'state'?: (string);
  'code'?: (string);
  'code_verifier'?: (string);
}

export interface HandleGoogleCallbackReq__Output {
  'state': (string);
  'code': (string);
  'code_verifier': (string);
}
