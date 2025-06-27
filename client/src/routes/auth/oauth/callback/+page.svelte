<!-- web-template/client/src/routes/auth/oauth/callback/+page.svelte -->

<script lang="ts">
	/**
	 * OAuth Callback Page
	 *
	 * Handles the OAuth callback from Google and completes the authentication flow.
	 * Users are redirected here after authorizing the application with Google.
	 */

	import { onMount } from 'svelte';
	import { authStore, isAuthenticated } from '$lib/stores';
	import { _ } from 'svelte-i18n';

	// Get data from load function
	let { data } = $props();

	let status = $state<'loading' | 'success' | 'error'>('loading');
	let errorMessage = $state('');
	let isNewUser = $state(false);

	onMount(async () => {
		// If already authenticated, redirect to home
		if ($isAuthenticated) {
			window.location.href = '/';
		}

		// Get OAuth parameters from load function
		const { token, userId, email, error, isNewUser: isNew } = data;

		// Handle OAuth error
		if (error) {
			status = 'error';
			errorMessage = getErrorMessage(error);
			authStore.setError(errorMessage);
			return;
		}

		// Handle missing token (successful OAuth should have token)
		if (!token || !userId || !email) {
			status = 'error';
			errorMessage = 'Invalid OAuth response from server';
			authStore.setError(errorMessage);
			return;
		}

		try {
			// Set authentication data directly from redirect parameters
			isNewUser = isNew;

			// Store the token and user data
			authStore.loginSuccess(
				{
					id: userId,
					email: decodeURIComponent(email),
					provider: 'google',
					provider_user_id: '', // Not provided in redirect
					created_at: new Date().toISOString(),
					updated_at: new Date().toISOString()
				},
				token
			);

			status = 'success';

			// Need to fetch the payment status from the server
			// since OAuth callback doesn't include payment_required field yet
			let paymentRequired = false;
			try {
				const { getCurrentUser } = await import('$lib/services/apiAuth');
				const userData = await getCurrentUser();
				paymentRequired = userData.payment_required || false;

				// Update the auth store with payment status
				authStore.setPaymentRequired(paymentRequired);
			} catch (err) {
				console.error('Failed to fetch payment status:', err);
				// Continue with redirect anyway
			}

			// Redirect after a brief success message
			setTimeout(async () => {
				if (paymentRequired) {
					window.location.href = '/payment';
				} else {
					window.location.href = '/';
				}
			}, 2000);
		} catch (err) {
			status = 'error';
			errorMessage = err instanceof Error ? err.message : 'OAuth authentication failed';
			console.error('OAuth callback error:', err);
		}
	});

	/**
	 * Get user-friendly error message
	 */
	function getErrorMessage(error: string): string {
		switch (error) {
			case 'no_invite':
				return $_('auth.errors.inviteOnly');
			case 'oauth_exchange_failed':
				return $_('auth.errors.exchangeCode');
			case 'invite_check_failed':
				return $_('auth.errors.verifyInvite');
			case 'user_creation_failed':
				return $_('auth.errors.createAccount');
			case 'user_lookup_failed':
				return $_('auth.errors.verifyAccount');
			case 'token_generation_failed':
				return $_('auth.errors.generateToken');
			default:
				return `OAuth error: ${error}`;
		}
	}

	/**
	 * Handle manual redirect to login
	 */
	async function goToLogin() {
		window.location.href = '/login';
	}
</script>

<svelte:head>
	<title>{$_('auth.oauth.pageTitle')}</title>
	<meta name="description" content={$_('auth.oauth.pageDescription')} />
</svelte:head>

<div class="bg-bg-primary flex min-h-screen items-center justify-center px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md space-y-8">
		<div class="text-center">
			<h2 class="text-text-primary mt-6 text-3xl font-extrabold">
				{#if status === 'loading'}
					{$_('auth.oauth.completing')}
				{:else if status === 'success'}
					{$_('auth.oauth.successful')}
				{:else}
					{$_('auth.oauth.failed')}
				{/if}
			</h2>
		</div>

		{#if status === 'loading'}
			<!-- Loading State -->
			<div class="text-center">
				<div class="inline-flex items-center space-x-3">
					<svg
						class="text-color-primary h-8 w-8 animate-spin"
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
					>
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
						></circle>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
						></path>
					</svg>
					<span class="text-text-secondary text-lg">{$_('auth.processing')}</span>
				</div>
				<p class="text-text-tertiary mt-4 text-sm">
					{$_('auth.pleaseWait')}
				</p>
			</div>
		{:else if status === 'success'}
			<!-- Success State -->
			<div class="text-center">
				<div
					class="bg-color-success-background mx-auto flex h-16 w-16 items-center justify-center rounded-full"
				>
					<svg
						class="text-color-success h-8 w-8"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"
						></path>
					</svg>
				</div>
				<h3 class="text-text-primary mt-4 text-lg font-medium">
					{#if isNewUser}
						{$_('auth.welcomeNew')}
					{:else}
						{$_('auth.welcomeBack')}
					{/if}
				</h3>
				<p class="text-text-secondary mt-2 text-sm">
					{#if isNewUser}
						{$_('auth.signupSuccess')}
					{:else}
						{$_('auth.signinSuccess')}
					{/if}
				</p>
			</div>
		{:else}
			<!-- Error State -->
			<div class="text-center">
				<div
					class="bg-color-error-background mx-auto flex h-16 w-16 items-center justify-center rounded-full"
				>
					<svg
						class="text-color-error h-8 w-8"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						></path>
					</svg>
				</div>
				<h3 class="text-text-primary mt-4 text-lg font-medium">{$_('auth.failed.title')}</h3>
				<p class="text-text-secondary mt-2 text-sm">{errorMessage}</p>

				<div class="mt-6">
					<button
						type="button"
						onclick={goToLogin}
						class="bg-color-primary text-text-on-primary hover:bg-color-primary-hover focus:ring-color-primary inline-flex items-center rounded-md border border-transparent px-4 py-2 text-sm font-medium shadow-sm transition-colors focus:ring-2 focus:ring-offset-2 focus:outline-none"
					>
						{$_('auth.failed.tryAgain')}
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>
