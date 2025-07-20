<script lang="ts">
	import type { PageProps } from './$types';
	import { decodeIdToken, type OAuth2Tokens } from 'arctic';
	import { AuthService, google } from '$lib/auth/service';
	import { goto } from '$app/navigation';
	import { UserService } from '$lib/user/service';
	let { data }: PageProps = $props();

	type Claims = {
		sub: string;
		name: string;
		picture: string;
		email: string;
	};

	$effect(() => {
		const storedState = sessionStorage.getItem('oauth_state');
		const codeVerifier = sessionStorage.getItem('oauth_code_verifier');
		const code = data.code;
		const state = data.state;

		async function validateAuthorizationCode() {
			let userService = new UserService(fetch);
			let authService = new AuthService(fetch);

			if (storedState === null || codeVerifier === null || code === null || state === null) {
				console.error('Missing one of: storedState, codeVerifier, code, or state');
				return new Response('Missing data. Please restart the login process.', { status: 400 });
			}
			if (storedState !== state) {
				console.error('State mismatch.', { storedState, state });
				return new Response('Invalid state. Please restart the login process.', { status: 400 });
			}

			let tokens: OAuth2Tokens;
			try {
				tokens = await google.validateAuthorizationCode(code, codeVerifier);
			} catch (e) {
				console.error('Failed to validate authorization code', e);
				return new Response('Authorization failed. Please try again.', { status: 400 });
			}

			const claims = decodeIdToken(tokens.idToken()) as Partial<Claims>;
			const googleId = claims.sub ?? undefined;
			if (googleId === undefined) {
				console.error('Missing Google ID in token claims', claims);
				return new Response('Missing Google ID. Please try again.', { status: 400 });
			}

			const existingUser = await userService.getUserIdFromGoogleId(googleId);
			const userId = existingUser?.id ?? (await userService.createUser(googleId)).user?.id;

			if (!userId) {
				console.error('User ID is missing');
				return new Response('User not available. Please try again.', { status: 500 });
			}

			const session = await authService.createSession(userId);
			if (!session.token) {
				console.error('Failed to create session');
				return new Response('Session missing. Please try again.', { status: 500 });
			}

			// Store session and redirect
			localStorage.setItem('sessionToken', session.token);
			goto('/');
		}

		validateAuthorizationCode();

		return () => {};
	});
</script>
