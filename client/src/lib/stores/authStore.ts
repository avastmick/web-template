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
import type { AuthState, User } from '$lib/types/auth';

// Storage keys for localStorage
const TOKEN_STORAGE_KEY = 'auth_token';
const USER_STORAGE_KEY = 'auth_user';
const PAYMENT_REQUIRED_KEY = 'payment_required';

// Initial state
const initialState: AuthState = {
	user: null,
	token: null,
	isAuthenticated: false,
	isLoading: false,
	error: null,
	paymentRequired: false
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
					const storedToken = localStorage.getItem(TOKEN_STORAGE_KEY);
					const storedUser = localStorage.getItem(USER_STORAGE_KEY);
					const paymentRequired = localStorage.getItem(PAYMENT_REQUIRED_KEY) === 'true';

					console.log('[AuthStore] Init - checking localStorage:', {
						hasToken: !!storedToken,
						hasUser: !!storedUser,
						paymentRequired
					});

					if (storedToken && storedUser) {
						const user: User = JSON.parse(storedUser);
						console.log('[AuthStore] Init - found valid auth data, updating store');
						update((state) => ({
							...state,
							user,
							token: storedToken,
							isAuthenticated: true,
							error: null,
							paymentRequired
						}));
					} else {
						console.log('[AuthStore] Init - no auth data found in localStorage');
					}
					initialized = true;
					resolve();
				} catch (error) {
					console.error('[AuthStore] Failed to load auth data from localStorage:', error);
					// Only clear data if we actually had corrupted data
					const storedToken = localStorage.getItem(TOKEN_STORAGE_KEY);
					const storedUser = localStorage.getItem(USER_STORAGE_KEY);
					if (storedToken || storedUser) {
						console.error('[AuthStore] Clearing corrupted auth data');
						localStorage.removeItem(TOKEN_STORAGE_KEY);
						localStorage.removeItem(USER_STORAGE_KEY);
						localStorage.removeItem(PAYMENT_REQUIRED_KEY);
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

		// Set payment required status
		setPaymentRequired: (required: boolean) => {
			if (browser) {
				if (required) {
					localStorage.setItem(PAYMENT_REQUIRED_KEY, 'true');
				} else {
					localStorage.removeItem(PAYMENT_REQUIRED_KEY);
				}
			}
			update((state) => ({ ...state, paymentRequired: required }));
		},

		// Login success - store user and token
		loginSuccess: (user: User, token: string) => {
			console.log('[AuthStore] loginSuccess called', { userId: user.id, email: user.email });
			if (browser) {
				console.log('[AuthStore] Storing auth data to localStorage');
				localStorage.setItem(TOKEN_STORAGE_KEY, token);
				localStorage.setItem(USER_STORAGE_KEY, JSON.stringify(user));

				// Verify it was stored
				const verifyToken = localStorage.getItem(TOKEN_STORAGE_KEY);
				const verifyUser = localStorage.getItem(USER_STORAGE_KEY);
				console.log('[AuthStore] Verification:', {
					tokenStored: !!verifyToken,
					userStored: !!verifyUser,
					tokenMatches: verifyToken === token
				});
			}

			update((state) => {
				console.log('[AuthStore] Updating store state to authenticated');
				return {
					...state,
					user,
					token,
					isAuthenticated: true,
					isLoading: false,
					error: null
				};
			});
		},

		// Update user data (for profile updates)
		updateUser: (user: User) => {
			if (browser) {
				localStorage.setItem(USER_STORAGE_KEY, JSON.stringify(user));
			}

			update((state) => ({
				...state,
				user
			}));
		},

		// Logout - clear all auth data
		logout: () => {
			if (browser) {
				localStorage.removeItem(TOKEN_STORAGE_KEY);
				localStorage.removeItem(USER_STORAGE_KEY);
				localStorage.removeItem(PAYMENT_REQUIRED_KEY);
			}

			set(initialState);
		},

		// Reset to initial state (useful for testing)
		reset: () => {
			set(initialState);
		},

		// Get the current token
		getToken: (): string | null => {
			if (browser) {
				return localStorage.getItem(TOKEN_STORAGE_KEY);
			}
			return null;
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

// Derived store for payment required status
export const paymentRequired = derived(authStore, ($authStore) => $authStore.paymentRequired);
