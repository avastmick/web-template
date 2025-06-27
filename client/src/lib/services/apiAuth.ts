// web-template/client/src/lib/services/apiAuth.ts

/**
 * Auth API client module
 *
 * Handles authentication operations including login, registration, and JWT token management.
 * All auth operations update the global auth store automatically.
 */

import { authStore } from '$lib/stores';
import type { LoginRequest, RegisterRequest, LoginResponse, User } from '$lib/types/auth';

// API base URL - can be configured based on environment
const API_BASE_URL = `${window.location.protocol}//${window.location.hostname}:${
	import.meta.env.VITE_SERVER_PORT || window.location.port || '8081'
}`;

// Custom error class for API errors
export class ApiError extends Error {
	constructor(
		message: string,
		public status: number,
		public data?: AuthError
	) {
		super(message);
		this.name = 'ApiError';
	}
}

// Auth error response type
export interface AuthError {
	error: string;
}

export interface OAuthLoginResponse extends LoginResponse {
	is_new_user: boolean;
}

/**
 * Register a new user
 * @param data Registration data
 * @returns User data and JWT token on success
 */
export async function register(data: RegisterRequest): Promise<LoginResponse> {
	authStore.setLoading(true);
	authStore.clearError();

	try {
		const response = await fetch(`${API_BASE_URL}/api/auth/register`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(data)
		});

		if (!response.ok) {
			let errorData: AuthError;
			try {
				errorData = await response.json();
			} catch {
				errorData = { error: `HTTP ${response.status}: ${response.statusText}` };
			}

			// Don't update auth state on error
			throw new ApiError(
				errorData.error || `Registration failed with status ${response.status}`,
				response.status,
				errorData
			);
		}

		const registerResponse: LoginResponse = await response.json();

		// Note: We don't automatically log in users after registration
		// They need to go through login flow
		authStore.setLoading(false);

		return registerResponse;
	} catch (error) {
		authStore.setLoading(false);
		const errorMessage =
			error instanceof ApiError ? error.message : 'Registration failed. Please try again.';
		authStore.setError(errorMessage);
		throw error;
	}
}

/**
 * Login user with email and password
 * @param data Login credentials
 * @returns User data and JWT token on success
 */
export async function login(data: LoginRequest): Promise<LoginResponse> {
	authStore.setLoading(true);
	authStore.clearError();

	try {
		const response = await fetch(`${API_BASE_URL}/api/auth/login`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(data)
		});

		if (!response.ok) {
			let errorData: AuthError;
			try {
				errorData = await response.json();
			} catch {
				errorData = { error: `HTTP ${response.status}: ${response.statusText}` };
			}

			throw new ApiError(
				errorData.error || `Login failed with status ${response.status}`,
				response.status,
				errorData
			);
		}

		const loginResponse: LoginResponse = await response.json();

		// Update auth store with login success
		authStore.loginSuccess(loginResponse.user, loginResponse.token);

		// Set payment required status if provided
		if (loginResponse.payment_required !== undefined) {
			authStore.setPaymentRequired(loginResponse.payment_required);
		}

		return loginResponse;
	} catch (error) {
		authStore.setLoading(false);
		const errorMessage = error instanceof ApiError ? error.message : 'Login failed';
		authStore.setError(errorMessage);
		throw error;
	}
}

/**
 * Logout user - clears auth state and token
 */
export async function logout(): Promise<void> {
	authStore.logout();
	// Clear payment status cache
	const { clearPaymentStatusCache } = await import('$lib/utils/auth');
	clearPaymentStatusCache();
	// Redirect to login page
	window.location.href = '/login';
}

/**
 * Verify JWT token validity
 * @param token JWT token to verify
 * @returns User data if token is valid
 */
export async function verifyToken(token?: string): Promise<User> {
	const authToken = token || localStorage.getItem('auth_token');

	if (!authToken) {
		throw new ApiError('No token provided', 401);
	}

	const response = await fetch(`${API_BASE_URL}/api/auth/verify`, {
		method: 'GET',
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${authToken}`
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
			errorData.error || `Token verification failed with status ${response.status}`,
			response.status,
			errorData
		);
	}

	const user: User = await response.json();
	return user;
}

/**
 * Refresh auth state from token
 * Useful for checking auth status on app initialization
 */
export async function refreshAuth(): Promise<void> {
	const token = localStorage.getItem('auth_token');

	if (!token) {
		authStore.logout();
		return;
	}

	try {
		const user = await verifyToken(token);
		authStore.loginSuccess(user, token);
	} catch {
		// Token is invalid, clear auth state
		authStore.logout();
	}
}

/**
 * Get authorization header for API requests
 * @returns Authorization header object or empty object
 */
export function getAuthHeader(): { Authorization?: string } {
	const token = localStorage.getItem('auth_token');

	if (!token) {
		return {};
	}

	return {
		Authorization: `Bearer ${token}`
	};
}

/**
 * Check if user is authenticated
 * @returns true if user has a valid token
 */
export function isAuthenticated(): boolean {
	return !!localStorage.getItem('auth_token');
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
 * Initiate GitHub OAuth login flow
 * Redirects the user to the server's OAuth endpoint
 */
export function initiateGitHubOAuth(state?: string): void {
	const url = new URL(`${API_BASE_URL}/api/auth/oauth/github`);
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

/**
 * Get current user data including payment status
 * @returns User data with payment status
 */
export async function getCurrentUser(): Promise<LoginResponse> {
	const token = localStorage.getItem('auth_token');
	if (!token) {
		throw new ApiError('No authentication token', 401);
	}

	const response = await fetch(`${API_BASE_URL}/api/users/me`, {
		method: 'GET',
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${token}`
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
			errorData.error || `Failed to get user data with status ${response.status}`,
			response.status,
			errorData
		);
	}

	const userData: LoginResponse = await response.json();

	// Update auth store with latest data
	authStore.loginSuccess(userData.user, userData.token);
	if (userData.payment_required !== undefined) {
		authStore.setPaymentRequired(userData.payment_required);
	}

	return userData;
}
