// web-template/client/src/lib/stores/index.ts

/**
 * Re-exports for all stores to provide a clean import interface
 */

export {
	authStore,
	currentUser,
	isAuthenticated,
	authToken,
	isAuthLoading,
	authError,
	hasValidToken
} from './authStore';
