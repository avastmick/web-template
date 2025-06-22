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
	import { login } from '$lib/services/apiAuth';
	import { authStore, authError, isAuthLoading, isAuthenticated } from '$lib/stores';
	import { Container, Flex, Button, Input } from '$lib/components/ui/index.js';
	import { GoogleOAuthButton, GitHubOAuthButton } from '$lib/components/auth/index.js';
	import type { LoginRequest } from '$lib/types';
	import { _ } from 'svelte-i18n';

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
			successMessage = $_('auth.register.success') + ' ' + $_('auth.login.subtitle');
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
			emailError = $_('validation.required');
			return false;
		}

		if (!EMAIL_REGEX.test(email)) {
			emailError = $_('validation.email');
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
			passwordError = $_('validation.required');
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
</script>

<svelte:head>
	<title>{$_('auth.login.pageTitle')}</title>
	<meta name="description" content={$_('auth.login.pageDescription')} />
</svelte:head>

<main id="main-content" tabindex="-1">
	<Container class="py-16">
		<Flex direction="col" align="center" justify="center" class="min-h-[80vh]">
			<div class="w-full max-w-md">
				<Flex direction="col" align="center" gap="6" class="mb-8 text-center">
					<h1 class="text-text-primary text-3xl font-extrabold tracking-tight">
						{$_('auth.login.title')}
					</h1>
					<p class="text-text-secondary">
						{$_('auth.login.subtitle')}
					</p>
					<p class="text-text-secondary">
						{$_('auth.login.noAccount')}
						<a
							href="/register"
							class="text-primary focus-visible-ring rounded-sm font-medium hover:underline"
						>
							{$_('auth.login.signUp')}
						</a>
					</p>
				</Flex>

				<!-- Success Message -->
				{#if showSuccessMessage}
					<div class="bg-color-success-background border-color-success mb-6 rounded-md border p-4">
						<Flex align="center" gap="3">
							<div class="flex-shrink-0">
								<svg
									class="text-color-success h-5 w-5"
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
							<p class="text-color-success text-sm font-medium">{successMessage}</p>
						</Flex>
					</div>
				{/if}

				<form class="space-y-6" onsubmit={handleSubmit}>
					<Flex direction="col" gap="4">
						<!-- Email Field -->
						<Input
							id="email"
							type="email"
							label={$_('auth.login.email')}
							placeholder={$_('auth.login.email')}
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
							label={$_('auth.login.password')}
							placeholder={$_('auth.login.password')}
							bind:value={password}
							onblur={handlePasswordBlur}
							disabled={isSubmitting || $isAuthLoading}
							error={passwordError}
							required
							autocomplete="current-password"
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
								{$_('auth.login.or')}
							</span>
						</div>
					</div>

					<!-- OAuth Login Options -->
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
									<h3 class="text-sm font-medium text-red-800">{$_('auth.login.error')}</h3>
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
						loadingText={$_('auth.login.submit') + '...'}
						class="w-full"
					>
						{$_('auth.login.submit')}
					</Button>

					<!-- Additional Options -->
					<div class="text-center">
						<button
							type="button"
							class="text-primary focus-visible-ring rounded-sm text-sm font-medium underline hover:no-underline"
							onclick={() => {
								// TODO: Implement password reset functionality
								alert($_('auth.forgotPassword.success'));
							}}
						>
							{$_('auth.login.forgotPassword')}
						</button>
					</div>
				</form>
			</div>
		</Flex>
	</Container>
</main>
