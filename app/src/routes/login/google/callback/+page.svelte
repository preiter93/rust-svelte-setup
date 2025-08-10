<script lang="ts">
	import type { PageProps } from './$types';
	import { AuthService } from '$lib/auth/service';
	import { goto } from '$app/navigation';
	let { data }: PageProps = $props();

	$effect(() => {
		async function validateAuthorizationCode() {
			const code = data.code;
			const state = data.state;
			if (code === null || state === null) {
				return new Response('Missing data. Please restart the login process.', { status: 400 });
			}

			let authService = new AuthService(fetch);
			await authService.handleGoogleCallback(state, code);

			goto('/');
		}

		validateAuthorizationCode();

		return () => {};
	});
</script>
