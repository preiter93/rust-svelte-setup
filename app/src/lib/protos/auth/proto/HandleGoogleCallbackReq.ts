// Original file: /Users/philippreiter/Svelte/sveltekit-rust-setup/services/auth/api.proto


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
