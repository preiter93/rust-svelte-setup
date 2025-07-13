<script lang="ts">
	import { onMount } from 'svelte';
	import type { PageProps } from './$types';
	import { decodeIdToken, type OAuth2Tokens } from 'arctic';
	import { google } from '$lib/auth/service';
	import { goto } from '$app/navigation';

	let { data }: PageProps = $props();

	type Claims = {
		sub: string;
		name: string;
		picture: string;
		email: string;
	};

	onMount(() => {
		const storedState = sessionStorage.getItem('oauth_state');
		const codeVerifier = sessionStorage.getItem('oauth_code_verifier');
		const code = data.code;
		const state = data.state;

		async function validateAuthorizationCode() {
			// TODO: Proper error handling.
			if (storedState === null || codeVerifier === null || code === null || state === null) {
				console.log('Please restart the process.');
				return;
			}
			if (storedState !== state) {
				console.log('Please restart the process.');
				return;
			}

			let tokens: OAuth2Tokens;
			try {
				tokens = await google.validateAuthorizationCode(code, codeVerifier);
			} catch (e) {
				return new Response('Please restart the process.', {
					status: 400
				});
			}

			const claims = decodeIdToken(tokens.idToken()) as Partial<Claims>;

			const googleId = claims.sub ?? null;
			const name = claims.name ?? null;
			const picture = claims.picture ?? null;
			const email = claims.email ?? null;

			// TODO:
			// const existingUser = getUserFromGoogleId(googleId);
			// if (existingUser !== null) {
			// 	const sessionToken = generateSessionToken();
			// 	const session = createSession(sessionToken, existingUser.id);
			// 	setSessionTokenCookie(event, sessionToken, session.expiresAt);
			// 	return new Response(null, {
			// 		status: 302,
			// 		headers: {
			// 			Location: '/'
			// 		}
			// 	});
			// }
			//
			// const user = createUser(googleId, email, name, picture);
			// const sessionToken = generateSessionToken();
			// const session = createSession(sessionToken, user.id);
			// setSessionTokenCookie(event, sessionToken, session.expiresAt);
		}

		validateAuthorizationCode();

		goto('/');
		return () => console.log('destroyed');
	});
</script>

<p>callback</p>
