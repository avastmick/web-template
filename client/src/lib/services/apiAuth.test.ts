import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Use vi.hoisted for mock functions that are used inside vi.mock factories
const {
	mockSetLoading,
	mockClearError,
	mockSetError,
	mockHandleAuthResponse,
	mockLogout,
	mockGetAuthToken,
	mockGetPaymentUser,
	mockGetAuthHeader,
	mockIsAuthenticated
} = vi.hoisted(() => ({
	mockSetLoading: vi.fn(),
	mockClearError: vi.fn(),
	mockSetError: vi.fn(),
	mockHandleAuthResponse: vi.fn(),
	mockLogout: vi.fn(),
	mockGetAuthToken: vi.fn(),
	mockGetPaymentUser: vi.fn(),
	mockGetAuthHeader: vi.fn(),
	mockIsAuthenticated: vi.fn()
}));

// Mock $lib/stores before importing apiAuth
vi.mock('$lib/stores', () => ({
	authStore: {
		setLoading: mockSetLoading,
		clearError: mockClearError,
		setError: mockSetError,
		handleAuthResponse: mockHandleAuthResponse,
		logout: mockLogout
	}
}));

// Mock $lib/services/storageService
vi.mock('$lib/services/storageService', () => ({
	StorageService: {
		getAuthToken: mockGetAuthToken,
		getPaymentUser: mockGetPaymentUser,
		getAuthHeader: mockGetAuthHeader,
		isAuthenticated: mockIsAuthenticated
	}
}));

import {
	register,
	login,
	logout,
	verifyToken,
	refreshAuth,
	getAuthHeader,
	isAuthenticated,
	initiateGoogleOAuth,
	initiateGitHubOAuth,
	handleOAuthCallback,
	getCurrentUser,
	ApiError
} from './apiAuth';

describe('apiAuth', () => {
	const mockFetch = vi.fn();

	beforeEach(() => {
		vi.stubGlobal('fetch', mockFetch);
		mockFetch.mockReset();
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	describe('register', () => {
		const mockUser = {
			id: 'user-123',
			email: 'test@example.com',
			created_at: '2026-01-10T12:00:00Z',
			updated_at: '2026-01-10T12:00:00Z'
		};

		const mockAuthResponse = {
			auth_token: 'jwt-token',
			auth_user: mockUser,
			payment_user: {
				payment_required: false,
				has_valid_invite: false
			}
		};

		it('should register a new user successfully', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockAuthResponse)
			});

			const result = await register({ email: 'test@example.com', password: 'password123' });

			expect(mockSetLoading).toHaveBeenCalledWith(true);
			expect(mockClearError).toHaveBeenCalled();
			// Check the path and body, not the exact URL (since port depends on env)
			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('/api/auth/register'),
				{
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ email: 'test@example.com', password: 'password123' })
				}
			);
			expect(mockHandleAuthResponse).toHaveBeenCalledWith(mockAuthResponse);
			expect(mockSetLoading).toHaveBeenCalledWith(false);
			expect(result).toEqual(mockAuthResponse);
		});

		it('should handle registration failure', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 400,
				statusText: 'Bad Request',
				json: () => Promise.resolve({ error: 'Email already exists' })
			});

			await expect(
				register({ email: 'existing@example.com', password: 'password123' })
			).rejects.toThrow('Email already exists');

			expect(mockSetLoading).toHaveBeenCalledWith(false);
			expect(mockSetError).toHaveBeenCalledWith('Email already exists');
		});

		it('should handle network error during registration', async () => {
			mockFetch.mockRejectedValueOnce(new Error('Network error'));

			await expect(
				register({ email: 'test@example.com', password: 'password123' })
			).rejects.toThrow();

			expect(mockSetLoading).toHaveBeenCalledWith(false);
			expect(mockSetError).toHaveBeenCalledWith('Registration failed. Please try again.');
		});
	});

	describe('login', () => {
		const mockAuthResponse = {
			auth_token: 'jwt-token',
			auth_user: {
				id: 'user-123',
				email: 'test@example.com',
				created_at: '2026-01-10T12:00:00Z',
				updated_at: '2026-01-10T12:00:00Z'
			},
			payment_user: {
				payment_required: false,
				has_valid_invite: false
			}
		};

		it('should login successfully', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockAuthResponse)
			});

			const result = await login({ email: 'test@example.com', password: 'password123' });

			expect(mockSetLoading).toHaveBeenCalledWith(true);
			expect(mockClearError).toHaveBeenCalled();
			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('/api/auth/login'),
				{
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ email: 'test@example.com', password: 'password123' })
				}
			);
			expect(mockHandleAuthResponse).toHaveBeenCalledWith(mockAuthResponse);
			expect(result).toEqual(mockAuthResponse);
		});

		it('should handle invalid credentials', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 401,
				statusText: 'Unauthorized',
				json: () => Promise.resolve({ error: 'Invalid email or password' })
			});

			await expect(
				login({ email: 'test@example.com', password: 'wrong' })
			).rejects.toThrow('Invalid email or password');

			expect(mockSetLoading).toHaveBeenCalledWith(false);
			expect(mockSetError).toHaveBeenCalledWith('Invalid email or password');
		});

		it('should handle JSON parse error in error response', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 500,
				statusText: 'Internal Server Error',
				json: () => Promise.reject(new Error('Invalid JSON'))
			});

			await expect(
				login({ email: 'test@example.com', password: 'password123' })
			).rejects.toThrow('HTTP 500: Internal Server Error');
		});
	});

	describe('logout', () => {
		it('should clear auth state and redirect', async () => {
			// Save original location
			const originalLocation = window.location;

			// Mock location.href setter
			const locationMock = { ...originalLocation, href: '' };
			Object.defineProperty(window, 'location', {
				value: locationMock,
				writable: true
			});

			await logout();

			expect(mockLogout).toHaveBeenCalled();
			expect(locationMock.href).toBe('/login');

			// Restore
			Object.defineProperty(window, 'location', {
				value: originalLocation,
				writable: true
			});
		});
	});

	describe('verifyToken', () => {
		const mockUser = {
			id: 'user-123',
			email: 'test@example.com',
			created_at: '2026-01-10T12:00:00Z',
			updated_at: '2026-01-10T12:00:00Z'
		};

		it('should verify token and return user', async () => {
			mockGetAuthToken.mockReturnValue('valid-token');

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockUser)
			});

			const result = await verifyToken();

			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('/api/auth/verify'),
				{
					method: 'GET',
					headers: {
						'Content-Type': 'application/json',
						Authorization: 'Bearer valid-token'
					}
				}
			);
			expect(result).toEqual(mockUser);
		});

		it('should use provided token over stored token', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockUser)
			});

			await verifyToken('custom-token');

			expect(mockFetch).toHaveBeenCalledWith(
				expect.any(String),
				expect.objectContaining({
					headers: expect.objectContaining({
						Authorization: 'Bearer custom-token'
					})
				})
			);
		});

		it('should throw error if no token available', async () => {
			mockGetAuthToken.mockReturnValue(null);

			await expect(verifyToken()).rejects.toThrow('No token provided');
		});

		it('should throw error on invalid token', async () => {
			mockGetAuthToken.mockReturnValue('invalid-token');

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 401,
				statusText: 'Unauthorized',
				json: () => Promise.resolve({ error: 'Token expired' })
			});

			await expect(verifyToken()).rejects.toThrow('Token expired');
		});
	});

	describe('refreshAuth', () => {
		it('should refresh auth from stored token', async () => {
			const mockUser = {
				id: 'user-123',
				email: 'test@example.com',
				created_at: '2026-01-10T12:00:00Z',
				updated_at: '2026-01-10T12:00:00Z'
			};

			mockGetAuthToken.mockReturnValue('stored-token');
			mockGetPaymentUser.mockReturnValue({
				payment_required: false,
				has_valid_invite: false
			});

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockUser)
			});

			await refreshAuth();

			expect(mockHandleAuthResponse).toHaveBeenCalledWith({
				auth_token: 'stored-token',
				auth_user: mockUser,
				payment_user: {
					payment_required: false,
					has_valid_invite: false
				}
			});
		});

		it('should logout if no token available', async () => {
			mockGetAuthToken.mockReturnValue(null);

			await refreshAuth();

			expect(mockLogout).toHaveBeenCalled();
		});

		it('should logout if token is invalid', async () => {
			mockGetAuthToken.mockReturnValue('invalid-token');

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 401,
				statusText: 'Unauthorized',
				json: () => Promise.resolve({ error: 'Invalid token' })
			});

			await refreshAuth();

			expect(mockLogout).toHaveBeenCalled();
		});
	});

	describe('getAuthHeader', () => {
		it('should return auth header from StorageService', () => {
			mockGetAuthHeader.mockReturnValue({
				Authorization: 'Bearer test-token'
			});

			const result = getAuthHeader();

			expect(result).toEqual({ Authorization: 'Bearer test-token' });
		});
	});

	describe('isAuthenticated', () => {
		it('should delegate to StorageService', () => {
			mockIsAuthenticated.mockReturnValue(true);

			const result = isAuthenticated();

			expect(result).toBe(true);
			expect(mockIsAuthenticated).toHaveBeenCalled();
		});

		it('should return false when not authenticated', () => {
			mockIsAuthenticated.mockReturnValue(false);

			const result = isAuthenticated();

			expect(result).toBe(false);
		});
	});

	describe('initiateGoogleOAuth', () => {
		it('should redirect to Google OAuth endpoint', () => {
			const originalLocation = window.location;
			const locationMock = { ...originalLocation, href: '' };
			Object.defineProperty(window, 'location', { value: locationMock, writable: true });

			initiateGoogleOAuth();

			expect(locationMock.href).toContain('/api/auth/oauth/google');
			expect(locationMock.href).not.toContain('state=');

			Object.defineProperty(window, 'location', { value: originalLocation, writable: true });
		});

		it('should include state parameter if provided', () => {
			const originalLocation = window.location;
			const locationMock = { ...originalLocation, href: '' };
			Object.defineProperty(window, 'location', { value: locationMock, writable: true });

			initiateGoogleOAuth('custom-state');

			expect(locationMock.href).toContain('/api/auth/oauth/google');
			expect(locationMock.href).toContain('state=custom-state');

			Object.defineProperty(window, 'location', { value: originalLocation, writable: true });
		});
	});

	describe('initiateGitHubOAuth', () => {
		it('should redirect to GitHub OAuth endpoint', () => {
			const originalLocation = window.location;
			const locationMock = { ...originalLocation, href: '' };
			Object.defineProperty(window, 'location', { value: locationMock, writable: true });

			initiateGitHubOAuth();

			expect(locationMock.href).toContain('/api/auth/oauth/github');
			expect(locationMock.href).not.toContain('state=');

			Object.defineProperty(window, 'location', { value: originalLocation, writable: true });
		});

		it('should include state parameter if provided', () => {
			const originalLocation = window.location;
			const locationMock = { ...originalLocation, href: '' };
			Object.defineProperty(window, 'location', { value: locationMock, writable: true });

			initiateGitHubOAuth('custom-state');

			expect(locationMock.href).toContain('/api/auth/oauth/github');
			expect(locationMock.href).toContain('state=custom-state');

			Object.defineProperty(window, 'location', { value: originalLocation, writable: true });
		});
	});

	describe('handleOAuthCallback', () => {
		const mockAuthResponse = {
			auth_token: 'jwt-token',
			auth_user: {
				id: 'user-123',
				email: 'test@example.com',
				created_at: '2026-01-10T12:00:00Z',
				updated_at: '2026-01-10T12:00:00Z'
			},
			payment_user: {
				payment_required: false,
				has_valid_invite: false
			}
		};

		it('should handle OAuth callback successfully', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockAuthResponse)
			});

			const result = await handleOAuthCallback('auth-code', 'state-123');

			expect(mockSetLoading).toHaveBeenCalledWith(true);
			expect(mockClearError).toHaveBeenCalled();
			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('/api/auth/oauth/google/callback?code=auth-code&state=state-123'),
				expect.objectContaining({
					method: 'GET',
					headers: { 'Content-Type': 'application/json' }
				})
			);
			expect(mockHandleAuthResponse).toHaveBeenCalledWith(mockAuthResponse);
			expect(result).toEqual(mockAuthResponse);
		});

		it('should handle OAuth callback failure', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 400,
				statusText: 'Bad Request',
				json: () => Promise.resolve({ error: 'Invalid code' })
			});

			await expect(handleOAuthCallback('invalid-code')).rejects.toThrow('Invalid code');

			expect(mockSetLoading).toHaveBeenCalledWith(false);
			expect(mockSetError).toHaveBeenCalledWith('Invalid code');
		});
	});

	describe('getCurrentUser', () => {
		const mockAuthResponse = {
			auth_token: '',
			auth_user: {
				id: 'user-123',
				email: 'test@example.com',
				created_at: '2026-01-10T12:00:00Z',
				updated_at: '2026-01-10T12:00:00Z'
			},
			payment_user: {
				payment_required: false,
				has_valid_invite: false
			}
		};

		it('should get current user and preserve token', async () => {
			mockGetAuthToken.mockReturnValue('stored-token');

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockAuthResponse)
			});

			const result = await getCurrentUser();

			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('/api/users/me'),
				{
					method: 'GET',
					headers: {
						'Content-Type': 'application/json',
						Authorization: 'Bearer stored-token'
					}
				}
			);

			// Should preserve existing token since response has empty auth_token
			expect(mockHandleAuthResponse).toHaveBeenCalledWith({
				...mockAuthResponse,
				auth_token: 'stored-token'
			});
			expect(result.auth_token).toBe('stored-token');
		});

		it('should throw error if not authenticated', async () => {
			mockGetAuthToken.mockReturnValue(null);

			await expect(getCurrentUser()).rejects.toThrow('No authentication token');
		});

		it('should handle API error', async () => {
			mockGetAuthToken.mockReturnValue('token');

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 401,
				statusText: 'Unauthorized',
				json: () => Promise.resolve({ error: 'Session expired' })
			});

			await expect(getCurrentUser()).rejects.toThrow('Session expired');
		});
	});

	describe('ApiError', () => {
		it('should create an ApiError with correct properties', () => {
			const error = new ApiError('Test error', 404, { error: 'Not found' });

			expect(error.message).toBe('Test error');
			expect(error.status).toBe(404);
			expect(error.data).toEqual({ error: 'Not found' });
			expect(error.name).toBe('ApiError');
		});
	});
});
