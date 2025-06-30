import { AuthFlowManager } from '$lib/services/authFlowManager';

/**
 * Auth guard for protecting routes
 * Returns true if navigation should proceed, false otherwise
 */
export async function checkAuth(
	requireAuth: boolean = true,
	requirePayment: boolean = true
): Promise<boolean> {
	// AuthFlowManager.handleProtectedRoute always checks authentication,
	// so requireAuth parameter is effectively ignored but kept for backward compatibility
	if (!requireAuth) {
		// If auth is not required, just return true
		return true;
	}
	return AuthFlowManager.handleProtectedRoute(requirePayment);
}

/**
 * Check if authenticated user should be redirected from public pages
 */
export async function checkPublicRoute(): Promise<void> {
	return AuthFlowManager.handlePublicRoute();
}

/**
 * Root page redirect logic
 */
export async function handleRootRedirect(): Promise<void> {
	return AuthFlowManager.handleRootRedirect();
}
