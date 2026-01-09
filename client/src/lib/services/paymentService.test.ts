import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Use vi.hoisted for mock functions that are used inside vi.mock factories
const { mockStripe, mockElements, mockLoadStripe } = vi.hoisted(() => {
	const mockElements = {
		create: vi.fn()
	};
	const mockStripe = {
		elements: vi.fn().mockReturnValue(mockElements),
		confirmPayment: vi.fn(),
		retrievePaymentIntent: vi.fn()
	};
	return {
		mockStripe,
		mockElements,
		mockLoadStripe: vi.fn().mockResolvedValue(mockStripe)
	};
});

vi.mock('@stripe/stripe-js', () => ({
	loadStripe: mockLoadStripe
}));

// Mock apiAuth's ApiError before importing
vi.mock('./apiAuth', () => ({
	ApiError: class ApiError extends Error {
		constructor(
			message: string,
			public status: number,
			public data?: unknown
		) {
			super(message);
			this.name = 'ApiError';
		}
	}
}));

// Mock window.location
Object.defineProperty(globalThis, 'window', {
	value: {
		location: {
			protocol: 'http:',
			hostname: 'localhost',
			port: '8081'
		}
	},
	writable: true
});

describe('PaymentService', () => {
	const mockFetch = vi.fn();
	const API_BASE_URL = 'http://localhost:8081';

	beforeEach(() => {
		vi.stubGlobal('fetch', mockFetch);
		mockFetch.mockReset();
		vi.clearAllMocks();

		// Reset localStorage mock
		vi.mocked(localStorage.getItem).mockReturnValue('test-token');

		// Reset stripe mock
		mockLoadStripe.mockResolvedValue(mockStripe);
		mockStripe.elements.mockReturnValue(mockElements);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	describe('PaymentService class', () => {
		it('should throw error if Stripe fails to load', async () => {
			mockLoadStripe.mockResolvedValueOnce(null);

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			await expect(service.init()).rejects.toThrow();
		});

		it('should throw error if Stripe not initialized when getting stripe', async () => {
			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			expect(() => service.getStripe()).toThrow('Stripe not initialized');
		});

		it('should throw error if Stripe not initialized when creating elements', async () => {
			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			expect(() => service.createElements('pi_secret_123')).toThrow('Stripe not initialized');
		});

		it('should throw error if Stripe not initialized when confirming payment', async () => {
			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			await expect(
				service.confirmPayment(mockElements as unknown as never, 'http://localhost:5173/success')
			).rejects.toThrow('Stripe not initialized');
		});

		it('should throw error if Stripe not initialized when retrieving payment intent', async () => {
			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			await expect(service.retrievePaymentIntent('pi_secret_123')).rejects.toThrow(
				'Stripe not initialized'
			);
		});
	});

	describe('createPaymentIntent', () => {
		it('should create payment intent successfully', async () => {
			const mockResponse = {
				client_secret: 'pi_secret_123',
				payment_intent_id: 'pi_123'
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();
			const result = await service.createPaymentIntent(1000, 'usd');

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE_URL}/api/payment/create-intent`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				},
				body: JSON.stringify({ amount_cents: 1000, currency: 'usd' })
			});
			expect(result).toEqual(mockResponse);
		});

		it('should use default currency if not provided', async () => {
			const mockResponse = {
				client_secret: 'pi_secret_123',
				payment_intent_id: 'pi_123'
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();
			await service.createPaymentIntent(1000);

			expect(mockFetch).toHaveBeenCalledWith(
				expect.any(String),
				expect.objectContaining({
					body: JSON.stringify({ amount_cents: 1000, currency: 'usd' })
				})
			);
		});

		it('should throw error on API failure', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 400,
				statusText: 'Bad Request',
				json: () => Promise.resolve({ error: 'Invalid amount' })
			});

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			await expect(service.createPaymentIntent(0)).rejects.toThrow(
				'Payment intent creation failed: Invalid amount'
			);
		});

		it('should handle network error', async () => {
			mockFetch.mockRejectedValueOnce(new Error('Network error'));

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			await expect(service.createPaymentIntent(1000)).rejects.toThrow(
				'Payment intent creation failed: Unknown error'
			);
		});
	});

	describe('getPaymentStatus', () => {
		it('should get payment status successfully', async () => {
			const mockResponse = {
				has_active_payment: true,
				payment_status: 'active',
				payment_type: 'subscription',
				subscription_end_date: '2027-01-10T12:00:00Z'
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();
			const result = await service.getPaymentStatus();

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE_URL}/api/payment/status`, {
				method: 'GET',
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				}
			});
			expect(result).toEqual(mockResponse);
		});

		it('should return inactive status when no payment', async () => {
			const mockResponse = {
				has_active_payment: false
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();
			const result = await service.getPaymentStatus();

			expect(result.has_active_payment).toBe(false);
		});

		it('should throw error on API failure', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 401,
				statusText: 'Unauthorized',
				json: () => Promise.resolve({ error: 'Not authenticated' })
			});

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			await expect(service.getPaymentStatus()).rejects.toThrow(
				'Failed to get payment status: Not authenticated'
			);
		});
	});

	describe('apiRequest (private method via createPaymentIntent/getPaymentStatus)', () => {
		it('should not include auth header if no token', async () => {
			vi.mocked(localStorage.getItem).mockReturnValue(null);

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({ has_active_payment: false })
			});

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();
			await service.getPaymentStatus();

			expect(mockFetch).toHaveBeenCalledWith(
				expect.any(String),
				expect.objectContaining({
					headers: {
						'Content-Type': 'application/json'
					}
				})
			);
		});

		it('should handle JSON parse error in error response', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 500,
				statusText: 'Internal Server Error',
				json: () => Promise.reject(new Error('Invalid JSON'))
			});

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			await expect(service.getPaymentStatus()).rejects.toThrow(
				'Failed to get payment status: HTTP 500: Internal Server Error'
			);
		});
	});

	describe('paymentService singleton', () => {
		it('should export a singleton instance', async () => {
			const { paymentService, PaymentService } = await import('./paymentService');
			expect(paymentService).toBeInstanceOf(PaymentService);
		});
	});

	// Tests that require Stripe initialization - these test the Stripe interactions
	// when the env is properly configured. We test them by mocking the Stripe calls.
	describe('Stripe interactions (when initialized)', () => {
		it('should call Stripe elements with client secret', async () => {
			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			// Bypass the env check by directly setting the internal stripe instance
			// This simulates what happens after a successful init()
			(service as unknown as { stripe: typeof mockStripe }).stripe = mockStripe;

			const result = service.createElements('pi_secret_123');

			expect(mockStripe.elements).toHaveBeenCalledWith({
				clientSecret: 'pi_secret_123',
				appearance: { theme: 'stripe' }
			});
			expect(result).toBe(mockElements);
		});

		it('should create payment element with layout options', async () => {
			const mockPaymentElement = { mount: vi.fn() };
			mockElements.create.mockReturnValue(mockPaymentElement);

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			(service as unknown as { stripe: typeof mockStripe }).stripe = mockStripe;
			const elements = service.createElements('pi_secret_123');

			const result = service.createPaymentElement(elements);

			expect(mockElements.create).toHaveBeenCalledWith('payment', {
				layout: {
					type: 'tabs',
					defaultCollapsed: false,
					radios: true,
					spacedAccordionItems: false
				}
			});
			expect(result).toBe(mockPaymentElement);
		});

		it('should confirm payment with return URL', async () => {
			const mockResult = {
				paymentIntent: {
					id: 'pi_123',
					status: 'succeeded'
				}
			};
			mockStripe.confirmPayment.mockResolvedValue(mockResult);

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			(service as unknown as { stripe: typeof mockStripe }).stripe = mockStripe;
			const elements = service.createElements('pi_secret_123');

			const result = await service.confirmPayment(elements, 'http://localhost:5173/success');

			expect(mockStripe.confirmPayment).toHaveBeenCalledWith({
				elements,
				confirmParams: {
					return_url: 'http://localhost:5173/success'
				}
			});
			expect(result).toEqual(mockResult);
		});

		it('should return error on payment failure', async () => {
			const mockError = {
				error: {
					type: 'card_error',
					message: 'Your card was declined'
				}
			};
			mockStripe.confirmPayment.mockResolvedValue(mockError);

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			(service as unknown as { stripe: typeof mockStripe }).stripe = mockStripe;
			const elements = service.createElements('pi_secret_123');

			const result = await service.confirmPayment(elements, 'http://localhost:5173/success');

			expect(result.error).toBeDefined();
			expect(result.error?.message).toBe('Your card was declined');
		});

		it('should retrieve payment intent', async () => {
			const mockResult = {
				paymentIntent: {
					id: 'pi_123',
					status: 'succeeded',
					amount: 1000
				}
			};
			mockStripe.retrievePaymentIntent.mockResolvedValue(mockResult);

			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			(service as unknown as { stripe: typeof mockStripe }).stripe = mockStripe;

			const result = await service.retrievePaymentIntent('pi_secret_123');

			expect(mockStripe.retrievePaymentIntent).toHaveBeenCalledWith('pi_secret_123');
			expect(result).toEqual(mockResult);
		});

		it('should return Stripe instance after initialization', async () => {
			const { PaymentService } = await import('./paymentService');
			const service = new PaymentService();

			(service as unknown as { stripe: typeof mockStripe }).stripe = mockStripe;

			const result = service.getStripe();

			expect(result).toBe(mockStripe);
		});
	});
});
