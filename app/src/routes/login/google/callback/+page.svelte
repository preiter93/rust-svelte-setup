<script lang="ts">
	import { onMount } from 'svelte';
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

	onMount(() => {
		const storedState = sessionStorage.getItem('oauth_state');
		const codeVerifier = sessionStorage.getItem('oauth_code_verifier');
		const code = data.code;
		const state = data.state;

		async function validateAuthorizationCode() {
			let userService = new UserService(fetch);
			let authService = new AuthService(fetch);

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
			const googleId = claims.sub ?? undefined;

			const existingUser = await userService.getUserByGoogleId(googleId);
			const user = existingUser?.user ?? (await userService.createUser(googleId));

			if (!user.id) {
				return;
			}

			const session = await authService.createSession(user.id);
			if (session.token === undefined) {
				return;
			}
			localStorage.setItem('sessionToken', session.token);

			goto('/');
		}

		validateAuthorizationCode();

		return () => {};
	});
</script>

<p>callback</p>
