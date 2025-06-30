// web-template/client/src/lib/utils/auth.ts

/**
 * Authentication utilities for handling navigation and redirects
 */

import { get } from 'svelte/store';
import { browser } from '$app/environment';
import { isAuthenticated, authStore, paymentRequired as paymentRequiredStore } from '$lib/stores';
import { getCurrentUser } from '$lib/services/apiAuth';

// Session storage keys
const PAYMENT_STATUS_KEY = 'payment_status_checked';
const PAYMENT_REQUIRED_KEY = 'payment_required';

export interface AuthCheckOptions {
	requireAuth?: boolean;
	requirePayment?: boolean;
	redirectTo?: string;
	currentUrl?: URL;
}

/**
 * Check authentication status and handle redirects
 * This centralizes all authentication logic to prevent redirect loops
 * NOTE: Using window.location.href instead of goto() to avoid UI duplication issues
 *
 * Payment status is cached in sessionStorage to avoid repeated API calls
 */
export async function checkAuthAndRedirect(options: AuthCheckOptions = {}): Promise<boolean> {
	// Only run in browser
	if (!browser) return false;

	const { requireAuth = false, requirePayment = false, redirectTo, currentUrl } = options;

	console.log('[Auth] checkAuthAndRedirect called', {
		requireAuth,
		requirePayment,
		currentUrl: currentUrl?.pathname
	});

	// Wait for auth store to be initialized
	await authStore.waitForInit();

	// Get current state
	const authenticated = get(isAuthenticated);
	// Use provided URL or fall back to page store (which may not be available in all contexts)
	const currentPath = currentUrl ? currentUrl.pathname : browser ? window.location.pathname : '/';

	console.log('[Auth] Auth check result:', { authenticated, currentPath });

	// Define protected routes that require authentication
	const protectedRoutes = ['/chat', '/payment', '/payment/success', '/payment/cancel'];
	const publicRoutes = ['/login', '/register'];
	const isProtectedRoute = protectedRoutes.includes(currentPath);
	const isPublicRoute = publicRoutes.includes(currentPath);

	// Check payment status - use sessionStorage cache if available
	let paymentRequired = false;
	if (authenticated) {
		// Check if we have payment status in session storage
		const cachedStatus = sessionStorage.getItem(PAYMENT_STATUS_KEY);
		const cachedPaymentRequired = sessionStorage.getItem(PAYMENT_REQUIRED_KEY);

		if (cachedStatus === 'true' && cachedPaymentRequired !== null) {
			// Use cached payment status
			paymentRequired = cachedPaymentRequired === 'true';
		} else {
			// Fetch from server and cache for this session
			try {
				const userData = await getCurrentUser();
				// Use the new UnifiedAuthResponse format
				paymentRequired = userData.payment_user.payment_required || false;

				// Cache in session storage
				sessionStorage.setItem(PAYMENT_STATUS_KEY, 'true');
				sessionStorage.setItem(PAYMENT_REQUIRED_KEY, paymentRequired.toString());
			} catch (error) {
				console.error('Failed to get user data:', error);
				// If we can't get user data, assume they need to login again
				window.location.href = '/login';
				return false;
			}
		}
	}

	// Handle authentication checks
	if (requireAuth && !authenticated) {
		// User needs to be authenticated but isn't
		if (currentPath !== '/login') {
			window.location.href = '/login';
		}
		return false;
	}

	// Handle payment requirement checks
	if (authenticated && paymentRequired && requirePayment) {
		// User is authenticated but needs to pay
		if (currentPath !== '/payment') {
			window.location.href = '/payment';
		}
		return false;
	}

	// Handle redirects for authenticated users on public routes
	if (authenticated && isPublicRoute) {
		// User is authenticated but on a public route (login/register)
		if (paymentRequired) {
			window.location.href = '/payment';
		} else {
			window.location.href = redirectTo || '/chat';
		}
		return true;
	}

	// Handle redirects for unauthenticated users on protected routes
	if (!authenticated && isProtectedRoute) {
		// User is not authenticated but trying to access protected route
		window.location.href = '/login';
		return false;
	}

	// Special handling for root route
	if (currentPath === '/') {
		if (!authenticated) {
			window.location.href = '/login';
		} else if (paymentRequired) {
			window.location.href = '/payment';
		} else {
			window.location.href = '/chat';
		}
		return true;
	}

	// No redirect needed
	return true;
}

/**
 * Check if the current user has completed payment
 */
export function hasCompletedPayment(): boolean {
	const auth = get(authStore);
	const paymentReq = get(paymentRequiredStore);
	return auth.isAuthenticated && !paymentReq;
}

/**
 * Clear payment status cache (call on logout)
 */
export function clearPaymentStatusCache(): void {
	if (browser) {
		sessionStorage.removeItem(PAYMENT_STATUS_KEY);
		sessionStorage.removeItem(PAYMENT_REQUIRED_KEY);
		// Also clear OAuth state if present
		sessionStorage.removeItem('oauth_state');
	}
}

/**
 * Check if the current route requires authentication
 */
export function isProtectedRoute(path: string): boolean {
	const protectedRoutes = ['/chat', '/payment', '/payment/success', '/payment/cancel'];
	return protectedRoutes.includes(path);
}
