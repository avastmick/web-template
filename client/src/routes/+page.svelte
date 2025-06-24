<!-- web-template/client/src/routes/+page.svelte -->

<script lang="ts">
	import { isAuthenticated, currentUser } from '$lib/stores';
	import { Button, Container, Flex } from '$lib/components/ui/index.js';
	import { _ } from 'svelte-i18n';
	import ChatInterface from '$lib/components/chat/ChatInterface.svelte';

	// Redirect logic for non-authenticated users
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(() => {
		// If user is not authenticated, redirect to login after a brief delay
		// This allows the authentication state to stabilize
		const checkAuth = setTimeout(() => {
			if (!$isAuthenticated) {
				goto('/login');
			}
		}, 100);

		return () => clearTimeout(checkAuth);
	});
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
	<meta name="description" content={$_('chat.description')} />
</svelte:head>

{#if $isAuthenticated && $currentUser}
	<!-- Authenticated User: Show Chat Interface -->
	<ChatInterface />
{:else}
	<!-- Loading state or redirect for non-authenticated users -->
	<main
		id="main-content"
		tabindex="-1"
		class="flex h-[calc(100vh-4rem)] items-center justify-center"
	>
		<Container class="text-center">
			<Flex direction="col" align="center" gap="6">
				<!-- Loading spinner -->
				<div class="relative">
					<svg
						class="text-color-primary h-12 w-12 animate-spin"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
						/>
					</svg>
				</div>

				<div class="space-y-2">
					<h1 class="text-text-primary text-xl font-semibold">
						{$_('auth.checkingAuthentication')}
					</h1>
					<p class="text-text-secondary">
						{$_('auth.pleaseWait')}
					</p>
				</div>

				<!-- Fallback sign in button if redirect doesn't work -->
				<div class="mt-8">
					<Flex justify="center" gap="4" class="flex-wrap">
						<Button>
							<a href="/login" class="text-inherit no-underline">{$_('auth.login.submit')}</a>
						</Button>
						<Button variant="ghost">
							<a href="/register" class="text-inherit no-underline">{$_('home.createAccount')}</a>
						</Button>
					</Flex>
				</div>
			</Flex>
		</Container>
	</main>
{/if}
