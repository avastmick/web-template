<!-- web-template/client/src/routes/chat/+page.svelte -->

<script lang="ts">
	import { isAuthenticated, currentUser, authStore } from '$lib/stores';
	import { _ } from 'svelte-i18n';
	import ChatInterface from '$lib/components/chat/ChatInterface.svelte';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	// Check authentication and payment status
	onMount(() => {
		const checkAccess = setTimeout(async () => {
			if (!$isAuthenticated) {
				// Not authenticated - redirect to login
				await goto('/login');
			} else if ($authStore.paymentRequired) {
				// Authenticated but needs payment
				await goto('/payment');
			}
			// Otherwise, user has access - show chat
		}, 100);

		return () => clearTimeout(checkAccess);
	});
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
	<meta name="description" content={$_('chat.description')} />
</svelte:head>

{#if $isAuthenticated && $currentUser && !$authStore.paymentRequired}
	<!-- User has full access: Show Chat Interface -->
	<ChatInterface />
{:else}
	<!-- Loading state while checking access -->
	<main
		id="main-content"
		tabindex="-1"
		class="flex h-[calc(100vh-4rem)] items-center justify-center"
	>
		<div class="text-text-secondary animate-pulse">
			{$_('common.loading')}
		</div>
	</main>
{/if}
