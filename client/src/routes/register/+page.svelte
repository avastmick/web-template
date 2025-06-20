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
	import { Container, Flex, Button, Input } from '$lib/components/ui/index.js';
	import { GoogleOAuthButton, GitHubOAuthButton } from '$lib/components/auth/index.js';
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

<main id="main-content" tabindex="-1">
	<Container class="py-16">
		<Flex direction="col" align="center" justify="center" class="min-h-[80vh]">
			<div class="w-full max-w-md">
				<Flex direction="col" align="center" gap="6" class="mb-8 text-center">
					<h1 class="text-text-primary text-3xl font-extrabold tracking-tight">
						Create your account
					</h1>
					<p class="text-text-secondary">
						Or
						<a
							href="/login"
							class="text-primary focus-visible-ring rounded-sm font-medium hover:underline"
						>
							sign in to your existing account
						</a>
					</p>
				</Flex>

				<form class="space-y-6" onsubmit={handleSubmit}>
					<Flex direction="col" gap="4">
						<!-- Email Field -->
						<Input
							id="email"
							type="email"
							label="Email address"
							placeholder="Enter your email"
							bind:value={email}
							onblur={handleEmailBlur}
							disabled={isSubmitting || $isAuthLoading}
							error={emailError}
							required
							autocomplete="email"
						/>

						<!-- Password Field -->
						<Input
							id="password"
							type="password"
							label="Password"
							placeholder="Enter your password"
							bind:value={password}
							onblur={handlePasswordBlur}
							disabled={isSubmitting || $isAuthLoading}
							error={passwordError}
							required
							autocomplete="new-password"
							helpText={passwordError
								? undefined
								: 'Password must be at least 12 characters with uppercase, lowercase, and numbers'}
						/>

						<!-- Confirm Password Field -->
						<Input
							id="confirm-password"
							type="password"
							label="Confirm Password"
							placeholder="Confirm your password"
							bind:value={confirmPassword}
							onblur={handleConfirmPasswordBlur}
							disabled={isSubmitting || $isAuthLoading}
							error={confirmPasswordError}
							required
							autocomplete="new-password"
						/>
					</Flex>

					<!-- OR Divider -->
					<div class="relative">
						<div class="absolute inset-0 flex items-center">
							<div class="border-border-default w-full border-t"></div>
						</div>
						<div class="relative flex justify-center text-sm">
							<span
								class="text-text-secondary px-2"
								style="background-color: var(--color-background-primary);"
							>
								Or continue with
							</span>
						</div>
					</div>

					<!-- OAuth Registration Options -->
					<Flex direction="col" gap="3">
						<GoogleOAuthButton disabled={isSubmitting || $isAuthLoading} />
						<GitHubOAuthButton disabled={isSubmitting || $isAuthLoading} />
					</Flex>

					<!-- Error Display -->
					{#if $authError}
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
									<h3 class="text-sm font-medium text-red-800">Registration failed</h3>
									<p class="mt-1 text-sm text-red-700">{$authError}</p>
								</div>
							</Flex>
						</div>
					{/if}

					<!-- Submit Button -->
					<Button
						type="submit"
						disabled={isSubmitting || $isAuthLoading}
						loading={isSubmitting || $isAuthLoading}
						loadingText="Creating Account..."
						class="w-full"
					>
						Create Account
					</Button>
				</form>
			</div>
		</Flex>
	</Container>
</main>
