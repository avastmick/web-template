import type { PageLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import { get } from 'svelte/store';
import { isAuthenticated, authStore } from '$lib/stores';

export const load: PageLoad = async () => {
	// Get current auth state
	const authenticated = get(isAuthenticated);
	const auth = get(authStore);

	// Handle redirects based on authentication status
	if (!authenticated) {
		// Not authenticated - redirect to login
		redirect(302, '/login');
	} else if (auth.paymentRequired) {
		// Authenticated but needs payment
		redirect(302, '/payment');
	} else {
		// Authenticated with access - redirect to chat
		redirect(302, '/chat');
	}
};
