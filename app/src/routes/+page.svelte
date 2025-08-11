<script lang="ts">
	import { goto } from '$app/navigation';
	import { AuthService } from '$lib/auth/service';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	async function logout() {
		let authService = new AuthService(fetch);
		await authService.deleteSession();

		goto('/login');
	}
</script>

<div style="display: flex; flex-direction: column; height: 100vh; position: relative;">
	<div class="absolute right-4 top-4 flex items-center gap-4">
		<button
			onclick={logout}
			class="rounded-md bg-blue-500 px-4 py-2 text-white shadow transition hover:bg-blue-600"
		>
			Logout
		</button>
	</div>

	<div style="flex: 1; display: flex; justify-content: center; align-items: center;">
		<h1 style="font-size: 2rem;">Welcome, {data.user?.name}!</h1>
	</div>
	<p>{data.user?.id}</p>
</div>
