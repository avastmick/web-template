<!-- web-template/client/src/routes/login/+page.svelte -->

<script lang="ts">
	/**
	 * User Login Page
	 *
	 * Provides a form for existing users to sign in with email and password.
	 * Handles JWT token storage and redirects to protected pages after login.
	 */

	import { onMount, onDestroy } from 'svelte';
	import { login } from '$lib/services/apiAuth';
	import { authStore, authError, isAuthLoading } from '$lib/stores';
	import { Container, Flex, Button, Card, Alert, FormField } from '$lib/components/ui/index.js';
	import { GoogleOAuthButton, GitHubOAuthButton } from '$lib/components/auth/index.js';
	import type { LoginRequest } from '$lib/types';
	import { _ } from 'svelte-i18n';
	import { validateEmail } from '$lib/utils/validation';

	// Get data from load function
	let { data } = $props();

	// Form state
	let email = $state('');
	let password = $state('');

	// Validation state
	let emailError = $state('');
	let passwordError = $state('');
	let isSubmitting = $state(false);

	// UI state
	let showSuccessMessage = $state(false);
	let successMessage = $state('');

	// Check for registration success message and auth status
	onMount(async () => {
		if (data.registered) {
			showSuccessMessage = true;
			successMessage = $_('auth.register.success') + ' ' + $_('auth.login.subtitle');
		}

		// Check if already authenticated
		const { checkPublicRoute } = await import('$lib/guards/authGuard');
		await checkPublicRoute();
	});

	// Clean up component state on unmount
	onDestroy(() => {
		// Clear form fields
		email = '';
		password = '';
		emailError = '';
		passwordError = '';
		isSubmitting = false;
		showSuccessMessage = false;
		successMessage = '';

		// Clear any auth errors
		authStore.clearError();
	});

	/**
	 * Validate email format
	 */
	function validateEmailField(): boolean {
		const validation = validateEmail(email);
		emailError = validation.isValid ? '' : validation.error || $_('validation.email');
		return validation.isValid;
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
		const isEmailValid = validateEmailField();
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

			// Login successful - check where to redirect
			const { checkPublicRoute } = await import('$lib/guards/authGuard');
			await checkPublicRoute();
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
		if (email) validateEmailField();
	}

	function handlePasswordBlur() {
		if (password) validatePassword();
	}
</script>

<svelte:head>
	<title>{$_('auth.login.pageTitle')}</title>
	<meta name="description" content={$_('auth.login.pageDescription')} />
</svelte:head>

<Container class="py-16">
	<Flex direction="col" align="center" justify="center" class="min-h-[80vh]">
		<div class="w-full max-w-lg">
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
						class="text-primary duration-fast hover:text-primary-hover rounded-sm font-medium transition-colors focus:underline focus:ring-2 focus:ring-amber-400 focus:ring-offset-2 focus:outline-none"
					>
						{$_('auth.login.signUp')}
					</a>
				</p>
			</Flex>

			<Card variant="default" padding="lg">
				<!-- Success Message -->
				{#if showSuccessMessage}
					<div class="mb-6">
						<Alert variant="success" title={$_('common.success')} description={successMessage} />
					</div>
				{/if}

				<!-- Error Display -->
				{#if $authError}
					<div class="mb-6">
						<Alert variant="error" title={$_('auth.login.error')} description={$authError} />
					</div>
				{/if}

				<form class="space-y-6" onsubmit={handleSubmit}>
					<Flex direction="col" gap="4">
						<!-- Email Field -->
						<FormField
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
						<FormField
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
							<span class="bg-background-primary text-text-secondary px-3">
								{$_('auth.login.or')}
							</span>
						</div>
					</div>

					<!-- OAuth Login Options -->
					<Flex direction="col" gap="3">
						<GoogleOAuthButton disabled={isSubmitting || $isAuthLoading} />
						<GitHubOAuthButton disabled={isSubmitting || $isAuthLoading} />
					</Flex>

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
							class="text-primary duration-fast hover:text-primary-hover rounded-sm text-sm font-medium underline transition-colors hover:no-underline focus:ring-2 focus:ring-amber-400 focus:ring-offset-2 focus:outline-none"
							onclick={() => {
								// TODO: Implement password reset functionality
								alert($_('auth.forgotPassword.success'));
							}}
						>
							{$_('auth.login.forgotPassword')}
						</button>
					</div>
				</form>
			</Card>
		</div>
	</Flex>
</Container>
