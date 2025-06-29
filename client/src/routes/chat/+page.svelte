<!-- web-template/client/src/routes/chat/+page.svelte -->

<script lang="ts">
	import { onMount } from 'svelte';
	import { _ } from 'svelte-i18n';
	import ChatInterface from '$lib/components/chat/ChatInterface.svelte';

	// Check auth and payment status
	onMount(async () => {
		const { checkAuth } = await import('$lib/guards/authGuard');
		await checkAuth(true, true); // Require auth and payment
	});
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
	<meta name="description" content={$_('chat.description')} />
</svelte:head>

<!-- User has full access: Show Chat Interface -->
<!-- Auth checks are handled in the load function, so if we're here, user has access -->
<ChatInterface />
