<script lang="ts">
	import { goto } from '$app/navigation';
	import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';
	import { UserService } from '$lib/user/service';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	async function logout() {
		let userService = new UserService(fetch);
		await userService.logoutUser();

		goto('/login');
	}
</script>

<div style="display: flex; flex-direction: column; height: 100vh; position: relative;">
	<div class="absolute top-4 right-4 flex items-center gap-4">
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
