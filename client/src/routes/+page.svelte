<!-- web-template/client/src/routes/+page.svelte -->
<!-- Landing page - Handles redirects only, no UI -->

<script lang="ts">
	import { isAuthenticated, authStore } from '$lib/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(() => {
		// Immediate redirect based on authentication status
		if (!$isAuthenticated) {
			// Not authenticated - redirect to login
			goto('/login');
		} else if ($authStore.paymentRequired) {
			// Authenticated but needs payment
			goto('/payment');
		} else {
			// Authenticated with access - redirect to chat
			goto('/chat');
		}
	});
</script>

<!-- No UI - this page only handles redirects -->
