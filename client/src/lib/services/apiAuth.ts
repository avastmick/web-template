// web-template/client/src/lib/services/apiAuth.ts

/**
 * API service for authentication endpoints
 *
 * This service handles all HTTP calls to the server's auth endpoints:
 * - User registration
 * - User login
 * - Fetching current user profile
 *
 * It integrates with the authStore for state management.
 */

import { authStore } from '$lib/stores/authStore';
import type {
	LoginRequest,
	RegisterRequest,
	LoginResponse,
	OAuthLoginResponse,
	User,
	AuthError
} from '$lib/types';

// Configuration
// In production/Docker, client and server run on the same port
// In development, they run on separate ports (client on CLIENT_PORT, server on VITE_SERVER_PORT)
const SERVER_PORT = import.meta.env.VITE_SERVER_PORT || window.location.port || '8081';
const API_BASE_URL = `${window.location.protocol}//${window.location.hostname}:${SERVER_PORT}`;

/**
 * Custom error class for API errors
 */
export class ApiError extends Error {
	constructor(
		message: string,
		public status: number,
		public response?: AuthError
	) {
		super(message);
		this.name = 'ApiError';
	}
}

/**
 * Helper function to make HTTP requests with proper error handling
 */
async function apiRequest<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
	const url = `${API_BASE_URL}${endpoint}`;

	// Default headers - use Record for easier manipulation
	const headers: Record<string, string> = {
		'Content-Type': 'application/json'
	};

	// Add any existing headers from options
	if (options.headers) {
		if (options.headers instanceof Headers) {
			options.headers.forEach((value, key) => {
				headers[key] = value;
			});
		} else if (Array.isArray(options.headers)) {
			options.headers.forEach(([key, value]) => {
				headers[key] = value;
			});
		} else {
			Object.assign(headers, options.headers);
		}
	}

	// Add Authorization header if we have a token
	// We need to get the current value synchronously, so we'll use get() if available
	// or create a temporary subscription
	let currentToken: string | null = null;
	const unsubscribe = authStore.subscribe((state) => {
		currentToken = state.token;
	});
	unsubscribe(); // Immediately unsubscribe after getting the value

	if (currentToken) {
		headers.Authorization = `Bearer ${currentToken}`;
	}

	try {
		const response = await fetch(url, {
			...options,
			headers
		});

		// Handle non-2xx responses
		if (!response.ok) {
			let errorData: AuthError;
			try {
				errorData = await response.json();
			} catch {
				errorData = { error: `HTTP ${response.status}: ${response.statusText}` };
			}

			throw new ApiError(
				errorData.error || `Request failed with status ${response.status}`,
				response.status,
				errorData
			);
		}

		// Handle empty responses (like 204 No Content)
		if (response.status === 204) {
			return {} as T;
		}

		return await response.json();
	} catch (error) {
		if (error instanceof ApiError) {
			throw error;
		}

		// Network errors, parsing errors, etc.
		throw new ApiError(error instanceof Error ? error.message : 'An unknown error occurred', 0);
	}
}

/**
 * Register a new user
 */
export async function register(data: RegisterRequest): Promise<User> {
	authStore.setLoading(true);
	authStore.clearError();

	try {
		const user = await apiRequest<User>('/api/auth/register', {
			method: 'POST',
			body: JSON.stringify(data)
		});

		authStore.setLoading(false);
		return user;
	} catch (error) {
		authStore.setLoading(false);
		const errorMessage = error instanceof ApiError ? error.message : 'Registration failed';
		authStore.setError(errorMessage);
		throw error;
	}
}

/**
 * Log in an existing user
 */
export async function login(data: LoginRequest): Promise<LoginResponse> {
	authStore.setLoading(true);
	authStore.clearError();

	try {
		const response = await apiRequest<LoginResponse>('/api/auth/login', {
			method: 'POST',
			body: JSON.stringify(data)
		});

		// Update auth store with login success
		authStore.loginSuccess(response.user, response.token);

		return response;
	} catch (error) {
		authStore.setLoading(false);
		const errorMessage = error instanceof ApiError ? error.message : 'Login failed';
		authStore.setError(errorMessage);
		throw error;
	}
}

/**
 * Fetch the current user's profile
 * Requires authentication
 */
export async function fetchCurrentUser(): Promise<User> {
	authStore.setLoading(true);
	authStore.clearError();

	try {
		const user = await apiRequest<User>('/api/users/me', {
			method: 'GET'
		});

		// Update the user data in the store
		authStore.updateUser(user);
		authStore.setLoading(false);

		return user;
	} catch (error) {
		authStore.setLoading(false);

		// If we get a 401, the token is likely invalid - logout
		if (error instanceof ApiError && error.status === 401) {
			authStore.logout();
		}

		const errorMessage = error instanceof ApiError ? error.message : 'Failed to fetch user data';
		authStore.setError(errorMessage);
		throw error;
	}
}

/**
 * Log out the current user
 * This clears local state - there's no server endpoint for logout with JWT
 */
export function logout(): void {
	authStore.logout();
}

/**
 * Check if the current user's token is still valid by trying to fetch their profile
 * This is useful for checking auth status on app initialization
 */
export async function validateToken(): Promise<boolean> {
	try {
		await fetchCurrentUser();
		return true;
	} catch {
		// If fetch fails, token is invalid or expired
		authStore.logout();
		return false;
	}
}

/**
 * Initiate Google OAuth login flow
 * Redirects the user to the server's OAuth endpoint
 */
export function initiateGoogleOAuth(state?: string): void {
	const url = new URL(`${API_BASE_URL}/api/auth/oauth/google`);
	if (state) {
		url.searchParams.set('state', state);
	}

	// Redirect to OAuth endpoint
	window.location.href = url.toString();
}

/**
 * Handle OAuth callback (called when user returns from OAuth provider)
 * This should be called on the OAuth callback page
 */
export async function handleOAuthCallback(
	code: string,
	state?: string
): Promise<OAuthLoginResponse> {
	authStore.setLoading(true);
	authStore.clearError();

	try {
		const url = new URL(`${API_BASE_URL}/api/auth/oauth/google/callback`);
		url.searchParams.set('code', code);
		if (state) {
			url.searchParams.set('state', state);
		}

		const response = await fetch(url.toString(), {
			method: 'GET',
			headers: {
				'Content-Type': 'application/json'
			}
		});

		if (!response.ok) {
			let errorData: AuthError;
			try {
				errorData = await response.json();
			} catch {
				errorData = { error: `HTTP ${response.status}: ${response.statusText}` };
			}

			throw new ApiError(
				errorData.error || `OAuth callback failed with status ${response.status}`,
				response.status,
				errorData
			);
		}

		const oauthResponse: OAuthLoginResponse = await response.json();

		// Update auth store with login success
		authStore.loginSuccess(oauthResponse.user, oauthResponse.token);

		return oauthResponse;
	} catch (error) {
		authStore.setLoading(false);
		const errorMessage = error instanceof ApiError ? error.message : 'OAuth login failed';
		authStore.setError(errorMessage);
		throw error;
	}
}
