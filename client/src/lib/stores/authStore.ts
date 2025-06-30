// web-template/client/src/lib/stores/authStore.ts

/**
 * Svelte store for managing authentication state
 *
 * This store manages:
 * - Current user data
 * - JWT token
 * - Authentication status
 * - Loading states
 * - Error messages
 */

import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';
import type { AuthState, User, PaymentUser, UnifiedAuthResponse } from '$lib/types/auth';
import { StorageService } from '$lib/services/storageService';

// Removed storage keys - now using StorageService

// Initial state
const initialState: AuthState = {
	user: null,
	token: null,
	isAuthenticated: false,
	isLoading: false,
	error: null,
	paymentUser: null
};

// Track if the store has been initialized
let initialized = false;
let initializationPromise: Promise<void> | null = null;

// Create the writable store
function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>(initialState);

	return {
		subscribe,

		// Initialize the store (load from localStorage if available)
		init: () => {
			if (!browser) return Promise.resolve();

			// Return existing promise if already initializing
			if (initializationPromise) return initializationPromise;

			// Create initialization promise
			initializationPromise = new Promise<void>((resolve) => {
				console.log('[AuthStore] Starting initialization');
				try {
					const storedToken = StorageService.getAuthToken();
					const storedUser = StorageService.getAuthUser();
					const paymentUser = StorageService.getPaymentUser();

					console.log('[AuthStore] Init - checking storage:', {
						hasToken: !!storedToken,
						hasUser: !!storedUser,
						hasPaymentUser: !!paymentUser
					});

					if (storedToken && storedUser) {
						console.log('[AuthStore] Init - found valid auth data, updating store');
						update((state) => ({
							...state,
							user: storedUser,
							token: storedToken,
							isAuthenticated: true,
							error: null,
							paymentUser
						}));
					} else {
						console.log('[AuthStore] Init - no auth data found in storage');
					}
					initialized = true;
					resolve();
				} catch (error) {
					console.error('[AuthStore] Failed to load auth data from localStorage:', error);
					// Only clear data if we actually had corrupted data
					if (StorageService.isAuthenticated()) {
						console.error('[AuthStore] Clearing corrupted auth data');
						StorageService.clearAll();
					}
					initialized = true;
					resolve();
				}
			});

			return initializationPromise;
		},

		// Check if store is initialized
		isInitialized: () => initialized,

		// Wait for initialization
		waitForInit: () => initializationPromise || Promise.resolve(),

		// Set loading state
		setLoading: (loading: boolean) => {
			update((state) => ({ ...state, isLoading: loading }));
		},

		// Set error message
		setError: (error: string | null) => {
			update((state) => ({ ...state, error }));
		},

		// Clear error message
		clearError: () => {
			update((state) => ({ ...state, error: null }));
		},

		// Set payment user data
		setPaymentUser: (paymentUser: PaymentUser | null) => {
			if (browser && paymentUser) {
				StorageService.setPaymentUser(paymentUser);
			}
			update((state) => ({ ...state, paymentUser }));
		},

		// Handle unified auth response
		handleAuthResponse: (response: UnifiedAuthResponse) => {
			console.log('[AuthStore] handleAuthResponse called', {
				userId: response.auth_user.id,
				email: response.auth_user.email,
				paymentRequired: response.payment_user.payment_required
			});

			if (browser) {
				console.log('[AuthStore] Storing auth data');
				StorageService.setAuthToken(response.auth_token);
				StorageService.setAuthUser(response.auth_user);
				StorageService.setPaymentUser(response.payment_user);
			}

			update((state) => {
				console.log('[AuthStore] Updating store state to authenticated');
				return {
					...state,
					user: response.auth_user,
					token: response.auth_token,
					paymentUser: response.payment_user,
					isAuthenticated: true,
					isLoading: false,
					error: null
				};
			});
		},

		// Legacy login success - for backward compatibility
		loginSuccess: (user: User, token: string) => {
			console.warn('[AuthStore] loginSuccess is deprecated, use handleAuthResponse instead');
			// Create a minimal UnifiedAuthResponse for backward compatibility
			const response: UnifiedAuthResponse = {
				auth_token: token,
				auth_user: user,
				payment_user: {
					payment_required: false,
					has_valid_invite: false
				}
			};
			authStore.handleAuthResponse(response);
		},

		// Update user data (for profile updates)
		updateUser: (user: User) => {
			if (browser) {
				StorageService.setAuthUser(user);
			}

			update((state) => ({
				...state,
				user
			}));
		},

		// Logout - clear all auth data
		logout: () => {
			if (browser) {
				StorageService.clearAll();
			}

			set(initialState);
		},

		// Reset to initial state (useful for testing)
		reset: () => {
			set(initialState);
		},

		// Get the current token
		getToken: (): string | null => {
			return StorageService.getAuthToken();
		},

		// Check if payment user data needs refresh
		needsPaymentRefresh: (): boolean => {
			return StorageService.isPaymentUserStale();
		}
	};
}

// Export the auth store instance
export const authStore = createAuthStore();

// Derived stores for convenient access to specific state properties
export const currentUser = derived(authStore, ($authStore) => $authStore.user);
export const isAuthenticated = derived(authStore, ($authStore) => $authStore.isAuthenticated);
export const authToken = derived(authStore, ($authStore) => $authStore.token);
export const isAuthLoading = derived(authStore, ($authStore) => $authStore.isLoading);
export const authError = derived(authStore, ($authStore) => $authStore.error);

// Helper function to check if user has a valid token
export const hasValidToken = derived(authStore, ($authStore) => {
	return $authStore.token !== null && $authStore.user !== null;
});

// Derived store for payment user data
export const paymentUser = derived(authStore, ($authStore) => $authStore.paymentUser);

// Derived store for payment required status (backward compatibility)
export const paymentRequired = derived(
	authStore,
	($authStore) => $authStore.paymentUser?.payment_required ?? false
);

// Derived store for checking if payment is valid
export const hasValidPayment = derived(authStore, ($authStore) =>
	StorageService.isPaymentValid($authStore.paymentUser)
);
