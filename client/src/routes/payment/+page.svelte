<!-- Payment page for users who need to pay to access the service -->

<script lang="ts">
	import { onMount } from 'svelte';
	import { _ } from 'svelte-i18n';
	import { Container, Flex, Button } from '$lib/components/ui/index.js';
	import { authStore, isAuthLoading } from '$lib/stores';
	import { paymentService } from '$lib/services/paymentService';
	import type { StripeElements, StripePaymentElement } from '@stripe/stripe-js';

	let loading = true;
	let processing = false;
	let error = '';
	let stripe: ReturnType<typeof paymentService.getStripe> | null = null;
	let elements: StripeElements | null = null;
	let paymentElement: StripePaymentElement | null = null;
	let paymentElementContainer: HTMLDivElement;

	// Payment amount in cents (e.g., $10.00 = 1000 cents)
	const PAYMENT_AMOUNT_CENTS = 1000; // $10.00
	const PAYMENT_CURRENCY = 'usd';

	onMount(async () => {
		try {
			// Initialize Stripe
			await paymentService.init();
			stripe = paymentService.getStripe();

			// Create payment intent
			const { client_secret } = await paymentService.createPaymentIntent(
				PAYMENT_AMOUNT_CENTS,
				PAYMENT_CURRENCY
			);

			// Create Stripe Elements
			elements = paymentService.createElements(client_secret);

			// Create and mount payment element
			paymentElement = paymentService.createPaymentElement(elements);
			paymentElement.mount(paymentElementContainer);

			loading = false;
		} catch (err) {
			console.error('Failed to initialize payment:', err);
			error = err instanceof Error ? err.message : $_('payment.error.initialization');
			loading = false;
		}
	});

	async function handleSubmit() {
		if (!stripe || !elements || processing) {
			return;
		}

		processing = true;
		error = '';

		try {
			// Confirm the payment
			const { error: stripeError } = await paymentService.confirmPayment(
				elements,
				`${window.location.origin}/payment/success`
			);

			if (stripeError) {
				// Show error to customer
				error = stripeError.message || $_('payment.error.processing');
				processing = false;
			} else {
				// Payment succeeded, redirect will happen automatically
				// The confirmPayment method redirects to the return_url
			}
		} catch (err) {
			console.error('Payment failed:', err);
			error = err instanceof Error ? err.message : $_('payment.error.unexpected');
			processing = false;
		}
	}

	function handleCancel() {
		// Redirect to home or show cancellation message
		window.location.href = '/';
	}

	// Check if user is authenticated
	$: if (!$authStore.isAuthenticated && !$isAuthLoading) {
		window.location.href = '/login';
	}
</script>

<svelte:head>
	<title>{$_('payment.pageTitle')}</title>
	<meta name="description" content={$_('payment.pageDescription')} />
</svelte:head>

<main id="main-content" tabindex="-1">
	<Container class="py-16">
		<Flex direction="col" align="center" justify="center" class="min-h-[80vh]">
			<div class="w-full max-w-md">
				<Flex direction="col" align="center" gap="6" class="mb-8 text-center">
					<h1 class="text-text-primary text-3xl font-extrabold tracking-tight">
						{$_('payment.title')}
					</h1>
					<p class="text-text-secondary max-w-prose">
						{$_('payment.description')}
					</p>
					<p class="text-text-primary text-xl font-semibold">
						{$_('payment.amount', { values: { amount: '$10.00' } })}
					</p>
				</Flex>

				{#if loading}
					<Flex justify="center" class="py-8">
						<div class="text-text-secondary animate-pulse">
							{$_('payment.loading')}
						</div>
					</Flex>
				{:else}
					<form class="space-y-6" on:submit|preventDefault={handleSubmit}>
						<!-- Stripe Payment Element Container -->
						<div
							bind:this={paymentElementContainer}
							class="border-border-default bg-background-secondary rounded-lg border p-4"
						></div>

						{#if error}
							<div class="rounded-md border border-red-200 bg-red-50 p-4">
								<Flex align="center" gap="3">
									<div class="flex-shrink-0">
										<svg
											class="h-5 w-5 text-red-400"
											viewBox="0 0 20 20"
											fill="currentColor"
											aria-hidden="true"
										>
											<path
												fill-rule="evenodd"
												d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z"
												clip-rule="evenodd"
											/>
										</svg>
									</div>
									<div>
										<p class="text-sm text-red-700">{error}</p>
									</div>
								</Flex>
							</div>
						{/if}

						<Flex gap="3">
							<Button
								type="submit"
								disabled={processing || loading}
								loading={processing}
								loadingText={$_('payment.processing')}
								class="flex-1"
							>
								{$_('payment.submit')}
							</Button>
							<Button
								type="button"
								variant="outline"
								disabled={processing}
								onclick={handleCancel}
								class="flex-1"
							>
								{$_('payment.cancel')}
							</Button>
						</Flex>

						<p class="text-text-secondary mt-4 text-center text-sm">
							{$_('payment.secureNotice')}
						</p>
					</form>
				{/if}
			</div>
		</Flex>
	</Container>
</main>

<style>
	/* Additional styles for Stripe Elements can be added here if needed */
</style>
