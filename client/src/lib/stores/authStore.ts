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

// Initial state
const initialState: AuthState = {
	user: null,
	token: null,
	isAuthenticated: false,
	isLoading: false,
	error: null
};

// Create the writable store
function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>(initialState);

	return {
		subscribe,

		// Initialize the store (load from localStorage if available)
		init: () => {
			if (!browser) return;

			try {
				const storedToken = localStorage.getItem(TOKEN_STORAGE_KEY);
				const storedUser = localStorage.getItem(USER_STORAGE_KEY);

				if (storedToken && storedUser) {
					const user: User = JSON.parse(storedUser);
					update((state) => ({
						...state,
						user,
						token: storedToken,
						isAuthenticated: true,
						error: null
					}));
				}
			} catch (error) {
				console.error('Failed to load auth data from localStorage:', error);
				// Clear potentially corrupted data
				localStorage.removeItem(TOKEN_STORAGE_KEY);
				localStorage.removeItem(USER_STORAGE_KEY);
			}
		},

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

		// Login success - store user and token
		loginSuccess: (user: User, token: string) => {
			if (browser) {
				localStorage.setItem(TOKEN_STORAGE_KEY, token);
				localStorage.setItem(USER_STORAGE_KEY, JSON.stringify(user));
			}

			update((state) => ({
				...state,
				user,
				token,
				isAuthenticated: true,
				isLoading: false,
				error: null
			}));
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
			}

			set(initialState);
		},

		// Reset to initial state (useful for testing)
		reset: () => {
			set(initialState);
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
