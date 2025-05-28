<!-- web-template/client/src/routes/profile/+layout.svelte -->

<script lang="ts">
	/**
	 * Profile Layout - Authentication Guard
	 *
	 * This layout ensures that all routes under /profile/* require authentication.
	 * If the user is not authenticated, they will be redirected to the login page.
	 */

	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { authStore, isAuthenticated, hasValidToken } from '$lib/stores';
	import { validateToken } from '$lib/services/apiAuth';

	onMount(async () => {
		// Initialize auth store from localStorage
		authStore.init();

		// Check if we have a token and if it's valid
		if ($hasValidToken) {
			// Validate the token with the server
			const isValid = await validateToken();
			if (!isValid) {
				// Token is invalid, redirect to login
				await goto('/login');
				return;
			}
		} else {
			// No token, redirect to login
			await goto('/login');
			return;
		}
	});
</script>

<!-- Only render content if authenticated -->
{#if $isAuthenticated}
	<slot />
{:else}
	<div class="flex min-h-screen items-center justify-center bg-gray-50">
		<div class="text-center">
			<div class="mb-4">
				<svg
					class="mx-auto h-12 w-12 animate-spin text-gray-400"
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
			</div>
			<p class="text-gray-600">Checking authentication...</p>
		</div>
	</div>
{/if}
