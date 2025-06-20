<!-- web-template/client/src/routes/profile/+page.svelte -->

<script lang="ts">
	/**
	 * User Profile Page
	 *
	 * Protected page that displays the current user's profile information.
	 * Includes logout functionality and fetches fresh user data on load.
	 */

	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { fetchCurrentUser, logout } from '$lib/services/apiAuth';
	import { authStore, currentUser, isAuthenticated, isAuthLoading, authError } from '$lib/stores';
	import { Container, Flex, Button } from '$lib/components/ui/index.js';

	// Component state
	let isLoadingProfile = false;
	let showWelcome = false;

	onMount(async () => {
		// Check for welcome parameter
		const urlParams = new URLSearchParams($page.url.search);
		if (urlParams.get('welcome') === 'true') {
			showWelcome = true;
		}

		// If not authenticated, redirect to login
		if (!$isAuthenticated) {
			await goto('/login');
			return;
		}

		// If we have a user in store but want to fetch fresh data
		// (optional - we could just use the stored user data)
		if ($currentUser) {
			isLoadingProfile = true;
			try {
				await fetchCurrentUser();
			} catch (error) {
				console.error('Failed to fetch fresh user data:', error);
				// If fetch fails due to invalid token, user will be logged out automatically
			} finally {
				isLoadingProfile = false;
			}
		}
	});

	/**
	 * Handle logout
	 */
	async function handleLogout() {
		logout();
		await goto('/login');
	}

	/**
	 * Handle refresh profile data
	 */
	async function handleRefresh() {
		isLoadingProfile = true;
		authStore.clearError();

		try {
			await fetchCurrentUser();
		} catch (error) {
			console.error('Failed to refresh profile data:', error);
		} finally {
			isLoadingProfile = false;
		}
	}

	/**
	 * Format date for display
	 */
	function formatDate(dateString: string): string {
		try {
			return new Date(dateString).toLocaleDateString('en-US', {
				year: 'numeric',
				month: 'long',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch {
			return dateString;
		}
	}
</script>

<svelte:head>
	<title>Profile - User Dashboard</title>
	<meta name="description" content="View and manage your user profile" />
</svelte:head>

<!-- Redirect if not authenticated -->
{#if !$isAuthenticated}
	<main id="main-content" tabindex="-1">
		<Container class="py-16">
			<Flex direction="col" align="center" justify="center" class="min-h-[80vh]">
				<div class="text-center">
					<p class="text-text-secondary">Redirecting to login...</p>
				</div>
			</Flex>
		</Container>
	</main>
{:else}
	<main id="main-content" tabindex="-1">
		<Container class="py-12">
			<Flex direction="col" gap="6" class="mx-auto max-w-4xl">
				<!-- Header -->
				<div class="bg-bg-secondary border-border-default rounded-lg border p-6 shadow-sm">
					<Flex align="center" justify="between" class="mb-4">
						<div>
							<h1 class="text-text-primary text-2xl font-bold">User Profile</h1>
							<p class="text-text-secondary mt-1">Your account information and settings</p>
						</div>
						<Flex gap="3">
							<Button
								variant="outline"
								size="sm"
								onclick={handleRefresh}
								disabled={isLoadingProfile || $isAuthLoading}
								loading={isLoadingProfile}
								loadingText="Refreshing..."
							>
								<svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
									></path>
								</svg>
								Refresh
							</Button>
							<Button variant="destructive" size="sm" onclick={handleLogout}>
								<svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
									></path>
								</svg>
								Sign Out
							</Button>
						</Flex>
					</Flex>
				</div>

				<!-- Welcome Message -->
				{#if showWelcome}
					<div class="bg-color-success-background border-color-success rounded-lg border p-4">
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
							<div>
								<h3 class="text-color-success text-sm font-medium">Welcome to the application!</h3>
								<p class="text-color-success mt-1 text-sm">
									Your account has been successfully created using Google OAuth. You can now access
									all features of the application.
								</p>
							</div>
						</Flex>
					</div>
				{/if}

				<!-- Error Display -->
				{#if $authError}
					<div class="rounded-lg border border-red-200 bg-red-50 p-4">
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
								<h3 class="text-sm font-medium text-red-800">Error loading profile</h3>
								<p class="mt-1 text-sm text-red-700">{$authError}</p>
							</div>
						</Flex>
					</div>
				{/if}

				<!-- Profile Information -->
				{#if $currentUser}
					<div class="bg-bg-secondary border-border-default rounded-lg border p-6 shadow-sm">
						<h2 class="text-text-primary mb-4 text-lg font-semibold">Profile Information</h2>
						<dl class="grid grid-cols-1 gap-x-6 gap-y-4 sm:grid-cols-2">
							<div>
								<dt class="text-text-secondary text-sm font-medium">User ID</dt>
								<dd class="text-text-primary mt-1 font-mono text-sm">{$currentUser.id}</dd>
							</div>
							<div>
								<dt class="text-text-secondary text-sm font-medium">Email Address</dt>
								<dd class="text-text-primary mt-1 text-sm">{$currentUser.email}</dd>
							</div>
							<div>
								<dt class="text-text-secondary text-sm font-medium">Account Created</dt>
								<dd class="text-text-primary mt-1 text-sm">
									{formatDate($currentUser.created_at)}
								</dd>
							</div>
							<div>
								<dt class="text-text-secondary text-sm font-medium">Last Updated</dt>
								<dd class="text-text-primary mt-1 text-sm">
									{formatDate($currentUser.updated_at)}
								</dd>
							</div>
						</dl>
					</div>

					<!-- Account Status -->
					<div class="bg-bg-secondary border-border-default rounded-lg border p-6 shadow-sm">
						<h3 class="text-text-primary mb-4 text-lg font-semibold">Account Status</h3>
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
							<div>
								<p class="text-text-primary text-sm font-medium">Active Account</p>
								<p class="text-text-secondary text-sm">Your account is in good standing</p>
							</div>
						</Flex>
					</div>
				{:else if $isAuthLoading || isLoadingProfile}
					<!-- Loading State -->
					<div class="bg-bg-secondary border-border-default rounded-lg border p-8 shadow-sm">
						<Flex align="center" justify="center" gap="3">
							<svg
								class="text-text-secondary h-8 w-8 animate-spin"
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
							<span class="text-text-secondary">Loading profile...</span>
						</Flex>
					</div>
				{/if}
			</Flex>
		</Container>
	</main>
{/if}
