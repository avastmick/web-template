<!-- web-template/client/src/routes/register/+page.svelte -->

<script lang="ts">
	/**
	 * User Registration Page
	 *
	 * Provides a form for new users to create an account with email and password.
	 * Includes client-side validation that mirrors the server-side validation.
	 */

	import { register } from '$lib/services/apiAuth';
	import { authStore, authError, isAuthLoading } from '$lib/stores';
	import { Container, Flex, Button, Card, Alert, FormField } from '$lib/components/ui/index.js';
	import { GoogleOAuthButton, GitHubOAuthButton } from '$lib/components/auth/index.js';
	import { _ } from 'svelte-i18n';
	import { onDestroy, onMount } from 'svelte';
	import { AuthFlowManager } from '$lib/services/authFlowManager';
	import {
		validateEmail,
		validatePasswordStrength,
		validatePasswordMatch
	} from '$lib/utils/validation';

	import type { RegisterRequest } from '$lib/types';
	import type { UnifiedAuthResponse } from '$lib/types/auth';

	// Form state
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');

	// Validation state
	let emailError = $state('');
	let passwordError = $state('');
	let confirmPasswordError = $state('');
	let isSubmitting = $state(false);

	// Registration state
	let registrationDetails: UnifiedAuthResponse;

	// Check if already authenticated on mount
	onMount(async () => {
		await AuthFlowManager.handlePublicRoute();
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
	 * Validate password strength (matching server-side validation)
	 */
	function validatePasswordField(): boolean {
		const validation = validatePasswordStrength(password);
		passwordError = validation.isValid
			? ''
			: validation.error || $_('validation.passwordMinLength');
		return validation.isValid;
	}

	/**
	 * Validate password confirmation
	 */
	function validateConfirmPasswordField(): boolean {
		const validation = validatePasswordMatch(password, confirmPassword);
		confirmPasswordError = validation.isValid
			? ''
			: validation.error || $_('auth.register.passwordMismatch');
		return validation.isValid;
	}

	/**
	 * Handle form submission
	 */
	async function handleSubmit(event: Event) {
		event.preventDefault();

		// Clear any previous auth errors
		authStore.clearError();

		// Validate all fields
		const isEmailValid = validateEmailField();
		const isPasswordValid = validatePasswordField();
		const isConfirmPasswordValid = validateConfirmPasswordField();

		if (!isEmailValid || !isPasswordValid || !isConfirmPasswordValid) {
			return;
		}

		isSubmitting = true;

		try {
			const userData: RegisterRequest = {
				email: email.trim(),
				password
			};

			registrationDetails = await register(userData);
			// Registration now returns token immediately, so user is logged in
			if (registrationDetails) {
				console.info('Registered and logged in user:', registrationDetails.auth_user.email);

				// Auth store is already updated by register() via handleAuthResponse
				// Use AuthFlowManager to handle navigation
				await AuthFlowManager.handleAuthSuccess();
			}
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
		if (email) validateEmailField();
	}

	function handlePasswordBlur() {
		if (password) validatePasswordField();
	}

	function handleConfirmPasswordBlur() {
		if (confirmPassword) validateConfirmPasswordField();
	}

	// Clean up component state on unmount
	onDestroy(() => {
		// Clear form fields
		email = '';
		password = '';
		confirmPassword = '';
		emailError = '';
		passwordError = '';
		confirmPasswordError = '';
		isSubmitting = false;

		// Clear any auth errors
		authStore.clearError();
	});
</script>

<svelte:head>
	<title>{$_('auth.register.pageTitle')}</title>
	<meta name="description" content={$_('auth.register.pageDescription')} />
</svelte:head>

<Container class="py-16">
	<Flex direction="col" align="center" justify="center" class="min-h-[80vh]">
		<div class="w-full max-w-lg">
			<Flex direction="col" align="center" gap="6" class="mb-8 text-center">
				<h1 class="text-text-primary text-3xl font-extrabold tracking-tight">
					{$_('auth.register.title')}
				</h1>
				<p class="text-text-secondary">
					{$_('auth.register.or')}
					<a
						href="/login"
						class="text-primary duration-fast hover:text-primary-hover rounded-sm font-medium transition-colors focus:underline focus:ring-2 focus:ring-amber-400 focus:ring-offset-2 focus:outline-none"
					>
						{$_('auth.register.signIn')}
					</a>
				</p>
			</Flex>

			<Card variant="raised" padding="lg">
				<!-- Error Display -->
				{#if $authError}
					<div class="mb-6">
						<Alert variant="error" title={$_('auth.register.error')} description={$authError} />
					</div>
				{/if}

				<form class="space-y-6" onsubmit={handleSubmit}>
					<Flex direction="col" gap="4">
						<!-- Email Field -->
						<FormField
							id="email"
							type="email"
							label={$_('auth.register.email')}
							placeholder={$_('auth.register.emailPlaceholder')}
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
							label={$_('auth.register.password')}
							placeholder={$_('auth.register.passwordPlaceholder')}
							bind:value={password}
							onblur={handlePasswordBlur}
							disabled={isSubmitting || $isAuthLoading}
							error={passwordError}
							hint={!passwordError ? $_('validation.passwordHelp') : undefined}
							required
							autocomplete="new-password"
						/>

						<!-- Confirm Password Field -->
						<FormField
							id="confirm-password"
							type="password"
							label={$_('auth.register.confirmPassword')}
							placeholder={$_('auth.register.confirmPasswordPlaceholder')}
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
							<div class="border-border w-full border-t"></div>
						</div>
						<div class="relative flex justify-center text-sm">
							<span class="bg-surface-raised text-text-secondary px-3">
								{$_('auth.oauth.continueWith')}
							</span>
						</div>
					</div>

					<!-- OAuth Registration Options -->
					<Flex direction="col" gap="3">
						<GoogleOAuthButton disabled={isSubmitting || $isAuthLoading} />
						<GitHubOAuthButton disabled={isSubmitting || $isAuthLoading} />
					</Flex>

					<!-- Submit Button -->
					<Button
						type="submit"
						disabled={isSubmitting || $isAuthLoading}
						loading={isSubmitting || $isAuthLoading}
						loadingText={$_('auth.register.loadingText')}
						class="w-full"
					>
						{$_('auth.register.submit')}
					</Button>
				</form>
			</Card>
		</div>
	</Flex>
</Container>
