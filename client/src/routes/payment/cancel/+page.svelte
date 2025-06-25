<!-- Payment cancelled page -->

<script lang="ts">
	import { goto } from '$app/navigation';
	import { _ } from 'svelte-i18n';
	import { Container, Flex, Button } from '$lib/components/ui/index.js';
	import { authStore } from '$lib/stores';

	function handleRetry() {
		goto('/payment');
	}

	function handleLogout() {
		authStore.logout();
		goto('/');
	}
</script>

<svelte:head>
	<title>{$_('payment.cancel.pageTitle')}</title>
	<meta name="description" content={$_('payment.cancel.pageDescription')} />
</svelte:head>

<main id="main-content" tabindex="-1">
	<Container class="py-16">
		<Flex direction="col" align="center" justify="center" class="min-h-[80vh]">
			<div class="w-full max-w-md text-center">
				<Flex direction="col" align="center" gap="6">
					<div class="rounded-full bg-yellow-100 p-3">
						<svg
							class="h-8 w-8 text-yellow-600"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
							/>
						</svg>
					</div>

					<h1 class="text-text-primary text-3xl font-bold">
						{$_('payment.cancel.title')}
					</h1>

					<p class="text-text-secondary max-w-prose">
						{$_('payment.cancel.description')}
					</p>

					<Flex gap="3" class="mt-4">
						<Button onclick={handleRetry} size="lg">
							{$_('payment.cancel.retry')}
						</Button>
						<Button onclick={handleLogout} variant="outline" size="lg">
							{$_('payment.cancel.logout')}
						</Button>
					</Flex>

					<p class="text-text-secondary mt-8 text-sm">
						{$_('payment.cancel.helpText')}
					</p>
				</Flex>
			</div>
		</Flex>
	</Container>
</main>
