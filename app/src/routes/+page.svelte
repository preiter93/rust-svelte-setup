<script lang="ts">
	import { goto } from '$app/navigation';

	function logout() {
		// TODO: Delete session in backend.

		localStorage.removeItem('sessionToken');
		goto('/login');
	}
</script>

Welcome
<button onclick={logout}>Logout</button>
