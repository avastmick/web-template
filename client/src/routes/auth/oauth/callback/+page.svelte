<!-- web-template/client/src/routes/auth/oauth/callback/+page.svelte -->

<script lang="ts">
	/**
	 * OAuth Callback Page
	 *
	 * Handles the OAuth callback from Google and completes the authentication flow.
	 * Users are redirected here after authorizing the application with Google.
	 */

	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { authStore, isAuthenticated } from '$lib/stores';

	let status: 'loading' | 'success' | 'error' = 'loading';
	let errorMessage = '';
	let isNewUser = false;

	onMount(async () => {
		// If already authenticated, redirect to profile
		if ($isAuthenticated) {
			await goto('/profile');
			return;
		}

		// Get OAuth parameters from URL
		const urlParams = new URLSearchParams($page.url.search);
		const token = urlParams.get('token');
		const userId = urlParams.get('user_id');
		const email = urlParams.get('email');
		const error = urlParams.get('error');
		const isNewUserParam = urlParams.get('is_new_user');

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
			isNewUser = isNewUserParam === 'true';

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

			// Redirect to profile after a brief success message
			setTimeout(async () => {
				if (isNewUser) {
					await goto('/profile?welcome=true');
				} else {
					await goto('/profile');
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
				return 'Registration is by invitation only. Please contact an administrator.';
			case 'oauth_exchange_failed':
				return 'Failed to exchange authorization code. Please try again.';
			case 'invite_check_failed':
				return 'Unable to verify invitation status. Please try again.';
			case 'user_creation_failed':
				return 'Failed to create user account. Please try again.';
			case 'user_lookup_failed':
				return 'Unable to verify user account. Please try again.';
			case 'token_generation_failed':
				return 'Failed to generate authentication token. Please try again.';
			default:
				return `OAuth error: ${error}`;
		}
	}

	/**
	 * Handle manual redirect to login
	 */
	async function goToLogin() {
		await goto('/login');
	}
</script>

<svelte:head>
	<title>OAuth Authentication - Completing Sign In</title>
	<meta name="description" content="Completing OAuth authentication process" />
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md space-y-8">
		<div class="text-center">
			<h2 class="mt-6 text-3xl font-extrabold text-gray-900">
				{#if status === 'loading'}
					Completing Sign In...
				{:else if status === 'success'}
					Sign In Successful!
				{:else}
					Sign In Failed
				{/if}
			</h2>
		</div>

		{#if status === 'loading'}
			<!-- Loading State -->
			<div class="text-center">
				<div class="inline-flex items-center space-x-3">
					<svg
						class="h-8 w-8 animate-spin text-indigo-600"
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
					<span class="text-lg text-gray-600">Processing authentication...</span>
				</div>
				<p class="mt-4 text-sm text-gray-500">
					Please wait while we complete your sign in with Google.
				</p>
			</div>
		{:else if status === 'success'}
			<!-- Success State -->
			<div class="text-center">
				<div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-green-100">
					<svg class="h-8 w-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"
						></path>
					</svg>
				</div>
				<h3 class="mt-4 text-lg font-medium text-gray-900">
					{#if isNewUser}
						Welcome! Your account has been created.
					{:else}
						Welcome back!
					{/if}
				</h3>
				<p class="mt-2 text-sm text-gray-600">
					{#if isNewUser}
						You've successfully signed up with Google. Redirecting to your profile...
					{:else}
						You've successfully signed in with Google. Redirecting to your profile...
					{/if}
				</p>
			</div>
		{:else}
			<!-- Error State -->
			<div class="text-center">
				<div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-100">
					<svg class="h-8 w-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						></path>
					</svg>
				</div>
				<h3 class="mt-4 text-lg font-medium text-gray-900">Authentication Failed</h3>
				<p class="mt-2 text-sm text-gray-600">{errorMessage}</p>

				<div class="mt-6">
					<button
						type="button"
						on:click={goToLogin}
						class="inline-flex items-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none"
					>
						Try Again
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>
