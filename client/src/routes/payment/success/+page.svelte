<!-- Payment success page -->

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { _ } from 'svelte-i18n';
	import { Container, Flex, Button } from '$lib/components/ui/index.js';
	import { paymentService } from '$lib/services/paymentService';
	import { fetchCurrentUser } from '$lib/services/apiAuth';

	let loading = true;
	let paymentStatus: 'succeeded' | 'processing' | 'failed' | null = null;
	let error = '';

	onMount(async () => {
		// Get payment intent client secret from URL
		const clientSecret = $page.url.searchParams.get('payment_intent_client_secret');

		if (!clientSecret) {
			error = $_('payment.success.missingIntent');
			loading = false;
			return;
		}

		try {
			// Initialize Stripe if not already done
			await paymentService.init();

			// Retrieve the payment intent to check its status
			const { paymentIntent, error: retrieveError } =
				await paymentService.retrievePaymentIntent(clientSecret);

			if (retrieveError || !paymentIntent) {
				error = retrieveError?.message || $_('payment.success.retrieveError');
				loading = false;
				return;
			}

			paymentStatus = paymentIntent.status as 'succeeded' | 'processing' | 'failed';

			// If payment succeeded, refresh user data to update payment status
			if (paymentStatus === 'succeeded') {
				try {
					await fetchCurrentUser();
				} catch (err) {
					console.error('Failed to refresh user data:', err);
					// Non-critical error, payment still succeeded
				}
			}

			loading = false;
		} catch (err) {
			console.error('Failed to verify payment:', err);
			error = err instanceof Error ? err.message : $_('payment.success.verifyError');
			loading = false;
		}
	});

	function handleContinue() {
		// Redirect to main app
		goto('/');
	}
</script>

<svelte:head>
	<title>{$_('payment.success.pageTitle')}</title>
	<meta name="description" content={$_('payment.success.pageDescription')} />
</svelte:head>

<main id="main-content" tabindex="-1">
	<Container class="py-16">
		<Flex direction="col" align="center" justify="center" class="min-h-[80vh]">
			<div class="w-full max-w-md text-center">
				{#if loading}
					<Flex direction="col" align="center" gap="4">
						<div class="text-text-secondary animate-pulse">
							{$_('payment.success.verifying')}
						</div>
					</Flex>
				{:else if error}
					<Flex direction="col" align="center" gap="6">
						<div class="rounded-full bg-red-100 p-3">
							<svg
								class="h-8 w-8 text-red-600"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M6 18L18 6M6 6l12 12"
								/>
							</svg>
						</div>
						<h1 class="text-text-primary text-2xl font-bold">
							{$_('payment.success.errorTitle')}
						</h1>
						<p class="text-text-secondary">{error}</p>
						<Button onclick={() => goto('/payment')}>
							{$_('payment.success.tryAgain')}
						</Button>
					</Flex>
				{:else if paymentStatus === 'succeeded'}
					<Flex direction="col" align="center" gap="6">
						<div class="rounded-full bg-green-100 p-3">
							<svg
								class="h-8 w-8 text-green-600"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M5 13l4 4L19 7"
								/>
							</svg>
						</div>
						<h1 class="text-text-primary text-3xl font-bold">
							{$_('payment.success.title')}
						</h1>
						<p class="text-text-secondary max-w-prose">
							{$_('payment.success.description')}
						</p>
						<Button onclick={handleContinue} size="lg">
							{$_('payment.success.continue')}
						</Button>
					</Flex>
				{:else if paymentStatus === 'processing'}
					<Flex direction="col" align="center" gap="6">
						<div class="rounded-full bg-yellow-100 p-3">
							<svg class="h-8 w-8 animate-spin text-yellow-600" fill="none" viewBox="0 0 24 24">
								<circle
									class="opacity-25"
									cx="12"
									cy="12"
									r="10"
									stroke="currentColor"
									stroke-width="4"
								></circle>
								<path
									class="opacity-75"
									fill="currentColor"
									d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
								></path>
							</svg>
						</div>
						<h1 class="text-text-primary text-2xl font-bold">
							{$_('payment.success.processingTitle')}
						</h1>
						<p class="text-text-secondary max-w-prose">
							{$_('payment.success.processingDescription')}
						</p>
						<Button onclick={handleContinue} variant="outline">
							{$_('payment.success.continueAnyway')}
						</Button>
					</Flex>
				{:else}
					<Flex direction="col" align="center" gap="6">
						<div class="rounded-full bg-red-100 p-3">
							<svg
								class="h-8 w-8 text-red-600"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
								/>
							</svg>
						</div>
						<h1 class="text-text-primary text-2xl font-bold">
							{$_('payment.success.failedTitle')}
						</h1>
						<p class="text-text-secondary max-w-prose">
							{$_('payment.success.failedDescription')}
						</p>
						<Button onclick={() => goto('/payment')}>
							{$_('payment.success.tryAgain')}
						</Button>
					</Flex>
				{/if}
			</div>
		</Flex>
	</Container>
</main>
