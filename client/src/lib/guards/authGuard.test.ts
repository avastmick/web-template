import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock $app/environment
vi.mock('$app/environment', () => ({
	browser: true
}));

// Mock AuthFlowManager
const mockAuthFlowManager = {
	handleProtectedRoute: vi.fn(),
	handlePublicRoute: vi.fn(),
	handleRootRedirect: vi.fn()
};

vi.mock('$lib/services/authFlowManager', () => ({
	AuthFlowManager: mockAuthFlowManager
}));

// Import after mocks
const { checkAuth, checkPublicRoute, handleRootRedirect } = await import('./authGuard');

describe('authGuard', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('checkAuth', () => {
		describe('when requireAuth is true (default)', () => {
			it('should call AuthFlowManager.handleProtectedRoute with requirePayment true by default', async () => {
				mockAuthFlowManager.handleProtectedRoute.mockResolvedValue(true);

				const result = await checkAuth();

				expect(mockAuthFlowManager.handleProtectedRoute).toHaveBeenCalledWith(true);
				expect(result).toBe(true);
			});

			it('should call AuthFlowManager.handleProtectedRoute with requirePayment false when specified', async () => {
				mockAuthFlowManager.handleProtectedRoute.mockResolvedValue(true);

				const result = await checkAuth(true, false);

				expect(mockAuthFlowManager.handleProtectedRoute).toHaveBeenCalledWith(false);
				expect(result).toBe(true);
			});

			it('should return false when user is not authenticated', async () => {
				mockAuthFlowManager.handleProtectedRoute.mockResolvedValue(false);

				const result = await checkAuth();

				expect(result).toBe(false);
			});

			it('should return false when user needs payment', async () => {
				mockAuthFlowManager.handleProtectedRoute.mockResolvedValue(false);

				const result = await checkAuth(true, true);

				expect(result).toBe(false);
			});

			it('should return true when user is authenticated and payment is valid', async () => {
				mockAuthFlowManager.handleProtectedRoute.mockResolvedValue(true);

				const result = await checkAuth(true, true);

				expect(result).toBe(true);
			});
		});

		describe('when requireAuth is false', () => {
			it('should return true without calling AuthFlowManager', async () => {
				const result = await checkAuth(false);

				expect(mockAuthFlowManager.handleProtectedRoute).not.toHaveBeenCalled();
				expect(result).toBe(true);
			});

			it('should return true regardless of requirePayment', async () => {
				const result = await checkAuth(false, true);

				expect(mockAuthFlowManager.handleProtectedRoute).not.toHaveBeenCalled();
				expect(result).toBe(true);
			});
		});
	});

	describe('checkPublicRoute', () => {
		it('should call AuthFlowManager.handlePublicRoute', async () => {
			mockAuthFlowManager.handlePublicRoute.mockResolvedValue(undefined);

			await checkPublicRoute();

			expect(mockAuthFlowManager.handlePublicRoute).toHaveBeenCalled();
		});

		it('should handle authenticated users on public routes', async () => {
			mockAuthFlowManager.handlePublicRoute.mockResolvedValue(undefined);

			await expect(checkPublicRoute()).resolves.toBeUndefined();
		});
	});

	describe('handleRootRedirect', () => {
		it('should call AuthFlowManager.handleRootRedirect', async () => {
			mockAuthFlowManager.handleRootRedirect.mockResolvedValue(undefined);

			await handleRootRedirect();

			expect(mockAuthFlowManager.handleRootRedirect).toHaveBeenCalled();
		});

		it('should handle root page redirect logic', async () => {
			mockAuthFlowManager.handleRootRedirect.mockResolvedValue(undefined);

			await expect(handleRootRedirect()).resolves.toBeUndefined();
		});
	});
});
