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
	import { fetchCurrentUser, logout } from '$lib/services/apiAuth';
	import { authStore, currentUser, isAuthenticated, isAuthLoading, authError } from '$lib/stores';

	// Component state
	let isLoadingProfile = false;

	onMount(async () => {
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
	<div class="flex min-h-screen items-center justify-center bg-gray-50">
		<div class="text-center">
			<p class="text-gray-600">Redirecting to login...</p>
		</div>
	</div>
{:else}
	<div class="min-h-screen bg-gray-50 py-12">
		<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
			<div class="mx-auto max-w-3xl">
				<!-- Header -->
				<div class="bg-white shadow">
					<div class="px-4 py-5 sm:px-6">
						<div class="flex items-center justify-between">
							<div>
								<h1 class="text-lg leading-6 font-medium text-gray-900">User Profile</h1>
								<p class="mt-1 max-w-2xl text-sm text-gray-500">
									Your account information and settings
								</p>
							</div>
							<div class="flex space-x-3">
								<button
									type="button"
									on:click={handleRefresh}
									disabled={isLoadingProfile || $isAuthLoading}
									class="inline-flex items-center rounded-md border border-gray-300 bg-white px-3 py-2 text-sm leading-4 font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
								>
									{#if isLoadingProfile}
										<svg
											class="mr-2 h-4 w-4 animate-spin text-gray-400"
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
										Refreshing...
									{:else}
										<svg
											class="mr-2 h-4 w-4 text-gray-400"
											fill="none"
											stroke="currentColor"
											viewBox="0 0 24 24"
										>
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
											></path>
										</svg>
										Refresh
									{/if}
								</button>
								<button
									type="button"
									on:click={handleLogout}
									class="inline-flex items-center rounded-md border border-transparent bg-red-600 px-3 py-2 text-sm leading-4 font-medium text-white shadow-sm hover:bg-red-700 focus:ring-2 focus:ring-red-500 focus:ring-offset-2 focus:outline-none"
								>
									<svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
										></path>
									</svg>
									Sign Out
								</button>
							</div>
						</div>
					</div>
				</div>

				<!-- Error Display -->
				{#if $authError}
					<div class="mt-4 rounded-md bg-red-50 p-4">
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
								<h3 class="text-sm font-medium text-red-800">Error loading profile</h3>
								<div class="mt-2 text-sm text-red-700">
									<p>{$authError}</p>
								</div>
							</div>
						</div>
					</div>
				{/if}

				<!-- Profile Information -->
				{#if $currentUser}
					<div class="mt-4 bg-white shadow">
						<div class="px-4 py-5 sm:p-6">
							<dl class="grid grid-cols-1 gap-x-4 gap-y-6 sm:grid-cols-2">
								<div>
									<dt class="text-sm font-medium text-gray-500">User ID</dt>
									<dd class="mt-1 font-mono text-sm text-gray-900">{$currentUser.id}</dd>
								</div>
								<div>
									<dt class="text-sm font-medium text-gray-500">Email Address</dt>
									<dd class="mt-1 text-sm text-gray-900">{$currentUser.email}</dd>
								</div>
								<div>
									<dt class="text-sm font-medium text-gray-500">Account Created</dt>
									<dd class="mt-1 text-sm text-gray-900">
										{formatDate($currentUser.created_at)}
									</dd>
								</div>
								<div>
									<dt class="text-sm font-medium text-gray-500">Last Updated</dt>
									<dd class="mt-1 text-sm text-gray-900">
										{formatDate($currentUser.updated_at)}
									</dd>
								</div>
							</dl>
						</div>
					</div>

					<!-- Account Status -->
					<div class="mt-4 bg-white shadow">
						<div class="px-4 py-5 sm:p-6">
							<h3 class="text-base font-medium text-gray-900">Account Status</h3>
							<div class="mt-4">
								<div class="flex items-center">
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
										<p class="text-sm font-medium text-gray-900">Active Account</p>
										<p class="text-sm text-gray-500">Your account is in good standing</p>
									</div>
								</div>
							</div>
						</div>
					</div>
				{:else if $isAuthLoading || isLoadingProfile}
					<!-- Loading State -->
					<div class="mt-4 bg-white shadow">
						<div class="px-4 py-5 sm:p-6">
							<div class="flex items-center justify-center">
								<svg
									class="h-8 w-8 animate-spin text-gray-400"
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
								<span class="ml-2 text-gray-600">Loading profile...</span>
							</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
