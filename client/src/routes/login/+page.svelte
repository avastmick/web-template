<!-- web-template/client/src/routes/login/+page.svelte -->

<script lang="ts">
	/**
	 * User Login Page
	 *
	 * Provides a form for existing users to sign in with email and password.
	 * Handles JWT token storage and redirects to protected pages after login.
	 */

	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { login, initiateGoogleOAuth, initiateGitHubOAuth } from '$lib/services/apiAuth';
	import { authStore, authError, isAuthLoading, isAuthenticated } from '$lib/stores';
	import type { LoginRequest } from '$lib/types';

	// Form state
	let email = '';
	let password = '';

	// Validation state
	let emailError = '';
	let passwordError = '';
	let isSubmitting = false;

	// UI state
	let showSuccessMessage = false;
	let successMessage = '';

	// Email validation regex
	const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

	// Check for registration success message
	onMount(() => {
		const urlParams = new URLSearchParams($page.url.search);
		if (urlParams.get('registered') === 'true') {
			showSuccessMessage = true;
			successMessage = 'Account created successfully! Please sign in below.';
		}

		// If user is already authenticated, redirect to profile
		if ($isAuthenticated) {
			goto('/profile');
		}
	});

	/**
	 * Validate email format
	 */
	function validateEmail(): boolean {
		if (!email.trim()) {
			emailError = 'Email is required';
			return false;
		}

		if (!EMAIL_REGEX.test(email)) {
			emailError = 'Email must be a valid email address';
			return false;
		}

		emailError = '';
		return true;
	}

	/**
	 * Validate password
	 */
	function validatePassword(): boolean {
		if (!password) {
			passwordError = 'Password is required';
			return false;
		}

		passwordError = '';
		return true;
	}

	/**
	 * Handle form submission
	 */
	async function handleSubmit(event: Event) {
		event.preventDefault();

		// Clear any previous messages
		authStore.clearError();
		showSuccessMessage = false;

		// Validate fields
		const isEmailValid = validateEmail();
		const isPasswordValid = validatePassword();

		if (!isEmailValid || !isPasswordValid) {
			return;
		}

		isSubmitting = true;

		try {
			const loginData: LoginRequest = {
				email: email.trim(),
				password
			};

			await login(loginData);

			// Login successful - redirect to profile page
			await goto('/profile');
		} catch (error) {
			// Error is already handled by the auth service and stored in authStore
			console.error('Login failed:', error);
		} finally {
			isSubmitting = false;
		}
	}

	/**
	 * Handle real-time validation
	 */
	function handleEmailBlur() {
		if (email) validateEmail();
	}

	function handlePasswordBlur() {
		if (password) validatePassword();
	}

	/**
	 * Handle Google OAuth login
	 */
	function handleGoogleLogin() {
		// Generate a random state for CSRF protection
		const state = crypto.randomUUID();

		// Store state in session storage for validation later
		sessionStorage.setItem('oauth_state', state);

		// Initiate OAuth flow
		initiateGoogleOAuth(state);
	}

	/**
	 * Handle GitHub OAuth login
	 */
	function handleGitHubLogin() {
		// Generate a random state for CSRF protection
		const state = crypto.randomUUID();

		// Store state in session storage for validation later
		sessionStorage.setItem('oauth_state', state);

		// Initiate OAuth flow
		initiateGitHubOAuth(state);
	}
</script>

<svelte:head>
	<title>Sign In - Login to Your Account</title>
	<meta name="description" content="Sign in to your account to access your profile" />
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md space-y-8">
		<div>
			<h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
				Sign in to your account
			</h2>
			<p class="mt-2 text-center text-sm text-gray-600">
				Or
				<a href="/register" class="font-medium text-indigo-600 hover:text-indigo-500">
					create a new account
				</a>
			</p>
		</div>

		<!-- Success Message -->
		{#if showSuccessMessage}
			<div class="rounded-md bg-green-50 p-4">
				<div class="flex">
					<div class="flex-shrink-0">
						<svg
							class="h-5 w-5 text-green-400"
							viewBox="0 0 20 20"
							fill="currentColor"
							aria-hidden="true"
						>
							<path
								fill-rule="evenodd"
								d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.236 4.53L7.73 10.16a.75.75 0 00-1.06 1.06l2.25 2.25a.75.75 0 001.14-.094l3.75-5.25z"
								clip-rule="evenodd"
							/>
						</svg>
					</div>
					<div class="ml-3">
						<p class="text-sm font-medium text-green-800">{successMessage}</p>
					</div>
				</div>
			</div>
		{/if}

		<form class="mt-8 space-y-6" on:submit={handleSubmit}>
			<div class="space-y-4">
				<!-- Email Field -->
				<div>
					<label for="email" class="block text-sm font-medium text-gray-700"> Email address </label>
					<input
						id="email"
						name="email"
						type="email"
						autocomplete="email"
						required
						bind:value={email}
						on:blur={handleEmailBlur}
						disabled={isSubmitting || $isAuthLoading}
						class="relative mt-1 block w-full appearance-none rounded-md border border-gray-300 px-3 py-2 text-gray-900 placeholder-gray-500 focus:z-10 focus:border-indigo-500 focus:ring-indigo-500 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 sm:text-sm"
						class:border-red-500={emailError}
						placeholder="Enter your email"
					/>
					{#if emailError}
						<p class="mt-1 text-sm text-red-600">{emailError}</p>
					{/if}
				</div>

				<!-- Password Field -->
				<div>
					<label for="password" class="block text-sm font-medium text-gray-700"> Password </label>
					<input
						id="password"
						name="password"
						type="password"
						autocomplete="current-password"
						required
						bind:value={password}
						on:blur={handlePasswordBlur}
						disabled={isSubmitting || $isAuthLoading}
						class="relative mt-1 block w-full appearance-none rounded-md border border-gray-300 px-3 py-2 text-gray-900 placeholder-gray-500 focus:z-10 focus:border-indigo-500 focus:ring-indigo-500 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 sm:text-sm"
						class:border-red-500={passwordError}
						placeholder="Enter your password"
					/>
					{#if passwordError}
						<p class="mt-1 text-sm text-red-600">{passwordError}</p>
					{/if}
				</div>
			</div>

			<!-- OR Divider -->
			<div class="relative">
				<div class="absolute inset-0 flex items-center">
					<div class="w-full border-t border-gray-300"></div>
				</div>
				<div class="relative flex justify-center text-sm">
					<span class="bg-gray-50 px-2 text-gray-500">Or continue with</span>
				</div>
			</div>

			<!-- OAuth Login Options -->
			<div class="space-y-3">
				<button
					type="button"
					on:click={handleGoogleLogin}
					disabled={isSubmitting || $isAuthLoading}
					class="group relative flex w-full justify-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-500 hover:bg-gray-50 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
				>
					<svg class="mr-2 h-5 w-5" viewBox="0 0 24 24">
						<path
							fill="#4285F4"
							d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
						/>
						<path
							fill="#34A853"
							d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
						/>
						<path
							fill="#FBBC05"
							d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
						/>
						<path
							fill="#EA4335"
							d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
						/>
					</svg>
					Continue with Google
				</button>

				<button
					type="button"
					on:click={handleGitHubLogin}
					disabled={isSubmitting || $isAuthLoading}
					class="group relative flex w-full justify-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-500 hover:bg-gray-50 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
				>
					<svg class="mr-2 h-5 w-5" viewBox="0 0 24 24" fill="#333">
						<path
							d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"
						/>
					</svg>
					Continue with GitHub
				</button>
			</div>

			<!-- Error Display -->
			{#if $authError}
				<div class="rounded-md bg-red-50 p-4">
					<div class="flex">
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
						<div class="ml-3">
							<h3 class="text-sm font-medium text-red-800">Sign in failed</h3>
							<div class="mt-2 text-sm text-red-700">
								<p>{$authError}</p>
							</div>
						</div>
					</div>
				</div>
			{/if}

			<!-- Submit Button -->
			<div>
				<button
					type="submit"
					disabled={isSubmitting || $isAuthLoading}
					class="group relative flex w-full justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
				>
					{#if isSubmitting || $isAuthLoading}
						<svg
							class="mr-3 -ml-1 h-5 w-5 animate-spin text-white"
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
						>
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
						Signing In...
					{:else}
						Sign In
					{/if}
				</button>
			</div>

			<!-- Additional Options -->
			<div class="flex items-center justify-between">
				<div class="text-sm">
					<button
						type="button"
						class="font-medium text-indigo-600 underline hover:text-indigo-500"
						on:click={() => {
							// TODO: Implement password reset functionality
							alert('Password reset functionality coming soon!');
						}}
					>
						Forgot your password?
					</button>
				</div>
			</div>
		</form>
	</div>
</div>
