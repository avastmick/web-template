import { browser } from '$app/environment';
import { authStore } from '$lib/stores';
import { goto } from '$app/navigation';

/**
 * Auth guard for protecting routes
 * Returns true if navigation should proceed, false otherwise
 */
export async function checkAuth(requireAuth = true, requirePayment = true): Promise<boolean> {
	if (!browser) return true; // Allow navigation on server

	// Wait for auth store to initialize
	await authStore.waitForInit();

	const token = authStore.getToken();

	if (requireAuth && !token) {
		// User needs to be authenticated
		await goto('/login');
		return false;
	}

	if (token && requirePayment) {
		// Check payment status
		try {
			const { getCurrentUser } = await import('$lib/services/apiAuth');
			const userData = await getCurrentUser();

			if (userData.payment_required) {
				// User needs to pay
				await goto('/payment');
				return false;
			}
		} catch {
			// If getCurrentUser fails, redirect to login
			await goto('/login');
			return false;
		}
	}

	return true;
}

/**
 * Check if authenticated user should be redirected from public pages
 */
export async function checkPublicRoute(): Promise<void> {
	if (!browser) return;

	// Wait for auth store to initialize
	await authStore.waitForInit();

	const token = authStore.getToken();

	if (token) {
		// User is authenticated, check where to redirect
		try {
			const { getCurrentUser } = await import('$lib/services/apiAuth');
			const userData = await getCurrentUser();

			if (userData.payment_required) {
				await goto('/payment');
			} else {
				await goto('/chat');
			}
		} catch {
			// Stay on current page if check fails
		}
	}
}

/**
 * Root page redirect logic
 */
export async function handleRootRedirect(): Promise<void> {
	if (!browser) return;

	// Wait for auth store to initialize
	await authStore.waitForInit();

	const token = authStore.getToken();

	if (!token) {
		await goto('/login');
		return;
	}

	// User is authenticated, check payment status
	try {
		const { getCurrentUser } = await import('$lib/services/apiAuth');
		const userData = await getCurrentUser();

		if (userData.payment_required) {
			await goto('/payment');
		} else {
			await goto('/chat');
		}
	} catch {
		// If getCurrentUser fails, redirect to login
		await goto('/login');
	}
}
