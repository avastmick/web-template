<!-- web-template/client/src/routes/+page.svelte -->
<!-- Landing page - Handles redirects based on auth status -->

<script lang="ts">
	import { onMount } from 'svelte';

	onMount(async () => {
		const { handleRootRedirect } = await import('$lib/guards/authGuard');
		await handleRootRedirect();
	});
</script>

<!-- Loading state while checking auth -->
<div class="flex h-screen items-center justify-center">
	<div class="text-text-secondary animate-pulse">Loading...</div>
</div>
