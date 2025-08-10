<script lang="ts">
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	function logout() {
		// TODO: Invalidate session in backend.

		goto('/login');
	}
</script>

<div style="display: flex; flex-direction: column; height: 100vh; position: relative;">
	<div class="absolute top-4 right-4 flex items-center gap-4">
		{#if data.user?.picture}
			<img
				src={data.user.picture}
				alt="Profile"
				class="h-10 w-10 rounded-full object-cover shadow-md"
			/>
		{/if}
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
