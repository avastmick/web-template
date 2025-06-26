import type { PageLoad } from './$types';
import { get } from 'svelte/store';
import { isAuthenticated, authStore } from '$lib/stores';

export const load: PageLoad = async () => {
	// Get current auth state
	const authenticated = get(isAuthenticated);
	const auth = get(authStore);

	// Handle redirects based on authentication status
	if (!authenticated) {
		// Not authenticated - redirect to login
		window.location.href = '/login';
	} else if (auth.paymentRequired) {
		// Authenticated but needs payment
		window.location.href = '/payment';
	} else {
		// Authenticated with access - redirect to chat
		window.location.href = '/chat';
	}
};
