import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import type { User, PaymentUser, UnifiedAuthResponse } from '$lib/types/auth';

// Mock $app/environment before importing authStore
vi.mock('$app/environment', () => ({
	browser: true
}));

// Mock StorageService
const mockStorageService = {
	getAuthToken: vi.fn(),
	getAuthUser: vi.fn(),
	getPaymentUser: vi.fn(),
	setAuthToken: vi.fn(),
	setAuthUser: vi.fn(),
	setPaymentUser: vi.fn(),
	clearAll: vi.fn(),
	isAuthenticated: vi.fn(),
	isPaymentUserStale: vi.fn(),
	isPaymentValid: vi.fn()
};

vi.mock('$lib/services/storageService', () => ({
	StorageService: mockStorageService
}));

// Import after mocks are set up
const { authStore, currentUser, isAuthenticated, authToken, isAuthLoading, authError, paymentUser } =
	await import('./authStore');

describe('authStore', () => {
	const mockUser: User = {
		id: 'user-123',
		email: 'test@example.com',
		provider: 'email',
		created_at: '2024-01-01T00:00:00Z',
		updated_at: '2024-01-01T00:00:00Z'
	};

	const mockPaymentUser: PaymentUser = {
		payment_required: false,
		has_valid_invite: true
	};

	const mockAuthResponse: UnifiedAuthResponse = {
		auth_token: 'mock-token-123',
		auth_user: mockUser,
		payment_user: mockPaymentUser
	};

	beforeEach(() => {
		vi.clearAllMocks();
		authStore.reset();
		mockStorageService.getAuthToken.mockReturnValue(null);
		mockStorageService.getAuthUser.mockReturnValue(null);
		mockStorageService.getPaymentUser.mockReturnValue(null);
		mockStorageService.isAuthenticated.mockReturnValue(false);
	});

	describe('initial state', () => {
		it('should have correct initial state', () => {
			const state = get(authStore);

			expect(state.user).toBeNull();
			expect(state.token).toBeNull();
			expect(state.isAuthenticated).toBe(false);
			expect(state.isLoading).toBe(false);
			expect(state.error).toBeNull();
			expect(state.paymentUser).toBeNull();
		});

		it('should have derived stores with correct values', () => {
			expect(get(currentUser)).toBeNull();
			expect(get(isAuthenticated)).toBe(false);
			expect(get(authToken)).toBeNull();
			expect(get(isAuthLoading)).toBe(false);
			expect(get(authError)).toBeNull();
			expect(get(paymentUser)).toBeNull();
		});
	});

	describe('handleAuthResponse', () => {
		it('should update state with auth response data', () => {
			authStore.handleAuthResponse(mockAuthResponse);

			const state = get(authStore);
			expect(state.user).toEqual(mockUser);
			expect(state.token).toBe('mock-token-123');
			expect(state.isAuthenticated).toBe(true);
			expect(state.isLoading).toBe(false);
			expect(state.error).toBeNull();
			expect(state.paymentUser).toEqual(mockPaymentUser);
		});

		it('should store auth data in StorageService', () => {
			authStore.handleAuthResponse(mockAuthResponse);

			expect(mockStorageService.setAuthToken).toHaveBeenCalledWith('mock-token-123');
			expect(mockStorageService.setAuthUser).toHaveBeenCalledWith(mockUser);
			expect(mockStorageService.setPaymentUser).toHaveBeenCalledWith(mockPaymentUser);
		});

		it('should update derived stores', () => {
			authStore.handleAuthResponse(mockAuthResponse);

			expect(get(currentUser)).toEqual(mockUser);
			expect(get(isAuthenticated)).toBe(true);
			expect(get(authToken)).toBe('mock-token-123');
			expect(get(paymentUser)).toEqual(mockPaymentUser);
		});
	});

	describe('logout', () => {
		it('should clear all auth state', () => {
			// First set up authenticated state
			authStore.handleAuthResponse(mockAuthResponse);
			expect(get(isAuthenticated)).toBe(true);

			// Then logout
			authStore.logout();

			const state = get(authStore);
			expect(state.user).toBeNull();
			expect(state.token).toBeNull();
			expect(state.isAuthenticated).toBe(false);
			expect(state.paymentUser).toBeNull();
		});

		it('should call StorageService.clearAll', () => {
			authStore.handleAuthResponse(mockAuthResponse);
			authStore.logout();

			expect(mockStorageService.clearAll).toHaveBeenCalled();
		});

		it('should update derived stores after logout', () => {
			authStore.handleAuthResponse(mockAuthResponse);
			authStore.logout();

			expect(get(currentUser)).toBeNull();
			expect(get(isAuthenticated)).toBe(false);
			expect(get(authToken)).toBeNull();
		});
	});

	describe('setLoading', () => {
		it('should update isLoading state', () => {
			authStore.setLoading(true);
			expect(get(isAuthLoading)).toBe(true);

			authStore.setLoading(false);
			expect(get(isAuthLoading)).toBe(false);
		});
	});

	describe('setError and clearError', () => {
		it('should set error message', () => {
			authStore.setError('Test error message');
			expect(get(authError)).toBe('Test error message');
		});

		it('should clear error message', () => {
			authStore.setError('Test error');
			authStore.clearError();
			expect(get(authError)).toBeNull();
		});
	});

	describe('setPaymentUser', () => {
		it('should update payment user state', () => {
			const newPaymentUser: PaymentUser = {
				payment_required: true,
				has_valid_invite: false,
				payment_status: 'active'
			};

			authStore.setPaymentUser(newPaymentUser);

			expect(get(paymentUser)).toEqual(newPaymentUser);
		});

		it('should persist to storage', () => {
			const newPaymentUser: PaymentUser = {
				payment_required: true,
				has_valid_invite: false
			};

			authStore.setPaymentUser(newPaymentUser);

			expect(mockStorageService.setPaymentUser).toHaveBeenCalledWith(newPaymentUser);
		});
	});

	describe('updateUser', () => {
		it('should update user data', () => {
			authStore.handleAuthResponse(mockAuthResponse);

			const updatedUser: User = {
				...mockUser,
				email: 'updated@example.com'
			};

			authStore.updateUser(updatedUser);

			expect(get(currentUser)).toEqual(updatedUser);
		});

		it('should persist updated user to storage', () => {
			authStore.handleAuthResponse(mockAuthResponse);

			const updatedUser: User = {
				...mockUser,
				email: 'updated@example.com'
			};

			authStore.updateUser(updatedUser);

			expect(mockStorageService.setAuthUser).toHaveBeenCalledWith(updatedUser);
		});
	});

	describe('init', () => {
		it('should restore state from storage if data exists', async () => {
			mockStorageService.getAuthToken.mockReturnValue('stored-token');
			mockStorageService.getAuthUser.mockReturnValue(mockUser);
			mockStorageService.getPaymentUser.mockReturnValue(mockPaymentUser);

			await authStore.init();

			const state = get(authStore);
			expect(state.token).toBe('stored-token');
			expect(state.user).toEqual(mockUser);
			expect(state.isAuthenticated).toBe(true);
			expect(state.paymentUser).toEqual(mockPaymentUser);
		});

		it('should not update state if no stored data', async () => {
			mockStorageService.getAuthToken.mockReturnValue(null);
			mockStorageService.getAuthUser.mockReturnValue(null);

			await authStore.init();

			const state = get(authStore);
			expect(state.isAuthenticated).toBe(false);
			expect(state.token).toBeNull();
		});
	});

	describe('getToken', () => {
		it('should return token from StorageService', () => {
			mockStorageService.getAuthToken.mockReturnValue('test-token');

			const token = authStore.getToken();

			expect(token).toBe('test-token');
		});
	});

	describe('needsPaymentRefresh', () => {
		it('should return result from StorageService', () => {
			mockStorageService.isPaymentUserStale.mockReturnValue(true);

			expect(authStore.needsPaymentRefresh()).toBe(true);

			mockStorageService.isPaymentUserStale.mockReturnValue(false);

			expect(authStore.needsPaymentRefresh()).toBe(false);
		});
	});

	describe('reset', () => {
		it('should reset to initial state', () => {
			authStore.handleAuthResponse(mockAuthResponse);
			authStore.reset();

			const state = get(authStore);
			expect(state.user).toBeNull();
			expect(state.token).toBeNull();
			expect(state.isAuthenticated).toBe(false);
		});
	});
});
