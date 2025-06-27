// place files you want to import through the `$lib` alias in this folder.

// Re-exports for clean imports
// First export from types and stores
export * from './types';
export * from './stores';

// Then export from services, but exclude conflicting exports
export {
	// From apiAuth.ts
	ApiError,
	register,
	login,
	logout,
	verifyToken,
	refreshAuth,
	getAuthHeader,
	// isAuthenticated is already exported from stores
	initiateGoogleOAuth,
	initiateGitHubOAuth,
	handleOAuthCallback,
	getCurrentUser
	// AuthError and OAuthLoginResponse are already exported from types
} from './services/apiAuth';

// Export other services
export * from './services/aiClient';
export { paymentService } from './services/paymentService';
