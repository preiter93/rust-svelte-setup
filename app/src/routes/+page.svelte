<script lang="ts">
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	function logout() {
		// TODO: Delete session in backend.

		localStorage.removeItem('sessionToken');
		goto('/login');
	}
</script>

<div style="display: flex; flex-direction: column; height: 100vh; position: relative;">
	<button
		onclick={logout}
		class="absolute top-4 right-4 rounded-md bg-blue-500 px-4 py-2 text-white shadow transition hover:bg-blue-600"
	>
		Logout
	</button>

	<div style="flex: 1; display: flex; justify-content: center; align-items: center;">
		<h1 style="font-size: 2rem;">Welcome</h1>
	</div>
	<p>{data.user?.id}</p>
</div>
