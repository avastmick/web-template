<!-- web-template/client/src/routes/register/+page.svelte -->

<script lang="ts">
	/**
	 * User Registration Page
	 *
	 * Provides a form for new users to create an account with email and password.
	 * Includes client-side validation that mirrors the server-side validation.
	 */

	import { goto } from '$app/navigation';
	import { register } from '$lib/services/apiAuth';
	import { authStore, authError, isAuthLoading } from '$lib/stores';
	import type { RegisterRequest } from '$lib/types';

	// Form state
	let email = '';
	let password = '';
	let confirmPassword = '';

	// Validation state
	let emailError = '';
	let passwordError = '';
	let confirmPasswordError = '';
	let isSubmitting = false;

	// Email validation regex (matching server-side validation)
	const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

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
	 * Validate password strength (matching server-side validation)
	 */
	function validatePassword(): boolean {
		if (!password) {
			passwordError = 'Password is required';
			return false;
		}

		if (password.length < 12) {
			passwordError = 'Password must be at least 12 characters long';
			return false;
		}

		// Additional strength checks
		if (!/[a-z]/.test(password)) {
			passwordError = 'Password must contain at least one lowercase letter';
			return false;
		}

		if (!/[A-Z]/.test(password)) {
			passwordError = 'Password must contain at least one uppercase letter';
			return false;
		}

		if (!/[0-9]/.test(password)) {
			passwordError = 'Password must contain at least one number';
			return false;
		}

		passwordError = '';
		return true;
	}

	/**
	 * Validate password confirmation
	 */
	function validateConfirmPassword(): boolean {
		if (!confirmPassword) {
			confirmPasswordError = 'Please confirm your password';
			return false;
		}

		if (password !== confirmPassword) {
			confirmPasswordError = 'Passwords do not match';
			return false;
		}

		confirmPasswordError = '';
		return true;
	}

	/**
	 * Handle form submission
	 */
	async function handleSubmit(event: Event) {
		event.preventDefault();

		// Clear any previous auth errors
		authStore.clearError();

		// Validate all fields
		const isEmailValid = validateEmail();
		const isPasswordValid = validatePassword();
		const isConfirmPasswordValid = validateConfirmPassword();

		if (!isEmailValid || !isPasswordValid || !isConfirmPasswordValid) {
			return;
		}

		isSubmitting = true;

		try {
			const userData: RegisterRequest = {
				email: email.trim(),
				password
			};

			await register(userData);

			// Registration successful - redirect to login page
			await goto('/login?registered=true');
		} catch (error) {
			// Error is already handled by the auth service and stored in authStore
			console.error('Registration failed:', error);
		} finally {
			isSubmitting = false;
		}
	}

	/**
	 * Handle real-time validation as user types
	 */
	function handleEmailBlur() {
		if (email) validateEmail();
	}

	function handlePasswordBlur() {
		if (password) validatePassword();
	}

	function handleConfirmPasswordBlur() {
		if (confirmPassword) validateConfirmPassword();
	}
</script>

<svelte:head>
	<title>Register - Create Account</title>
	<meta name="description" content="Create a new account to get started" />
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md space-y-8">
		<div>
			<h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">Create your account</h2>
			<p class="mt-2 text-center text-sm text-gray-600">
				Or
				<a href="/login" class="font-medium text-indigo-600 hover:text-indigo-500">
					sign in to your existing account
				</a>
			</p>
		</div>

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
						autocomplete="new-password"
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
					{:else}
						<p class="mt-1 text-sm text-gray-500">
							Password must be at least 12 characters with uppercase, lowercase, and numbers
						</p>
					{/if}
				</div>

				<!-- Confirm Password Field -->
				<div>
					<label for="confirm-password" class="block text-sm font-medium text-gray-700">
						Confirm Password
					</label>
					<input
						id="confirm-password"
						name="confirm-password"
						type="password"
						autocomplete="new-password"
						required
						bind:value={confirmPassword}
						on:blur={handleConfirmPasswordBlur}
						disabled={isSubmitting || $isAuthLoading}
						class="relative mt-1 block w-full appearance-none rounded-md border border-gray-300 px-3 py-2 text-gray-900 placeholder-gray-500 focus:z-10 focus:border-indigo-500 focus:ring-indigo-500 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 sm:text-sm"
						class:border-red-500={confirmPasswordError}
						placeholder="Confirm your password"
					/>
					{#if confirmPasswordError}
						<p class="mt-1 text-sm text-red-600">{confirmPasswordError}</p>
					{/if}
				</div>
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
							<h3 class="text-sm font-medium text-red-800">Registration failed</h3>
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
						Creating Account...
					{:else}
						Create Account
					{/if}
				</button>
			</div>
		</form>
	</div>
</div>
