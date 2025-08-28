<script lang="ts">
	import type { PageProps } from './$types';
	import { AuthService } from '$lib/auth/service';
	import { goto } from '$app/navigation';
	let { data }: PageProps = $props();

	$effect(() => {
		async function validateAuthorizationCode() {
			const provider = data.provider;
			if (provider != 'google' && provider != 'github') {
				return new Response('Unsupported oauth provider: ' + provider, { status: 400 });
			}

			const code = data.code;
			const state = data.state;
			if (code === null || state === null) {
				return new Response('Missing data. Please restart the login process.', { status: 400 });
			}

			let authService = new AuthService(fetch);
			await authService.handleOauthCallback(data.provider, state, code);

			goto('/');
		}

		validateAuthorizationCode();

		return () => {};
	});
</script>
