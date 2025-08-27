// Original file: /Users/philippreiter/Svelte/rust-svelte-setup/services/user/api.proto

export const OauthProvider = {
  OAUTH_PROVIDER_UNSPECIFIED: 0,
  OAUTH_PROVIDER_GOOGLE: 1,
  OAUTH_PROVIDER_GITHUB: 2,
} as const;

export type OauthProvider =
  | 'OAUTH_PROVIDER_UNSPECIFIED'
  | 0
  | 'OAUTH_PROVIDER_GOOGLE'
  | 1
  | 'OAUTH_PROVIDER_GITHUB'
  | 2

export type OauthProvider__Output = typeof OauthProvider[keyof typeof OauthProvider]
