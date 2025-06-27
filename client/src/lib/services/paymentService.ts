// web-template/client/src/lib/services/paymentService.ts

/**
 * Payment service for handling Stripe payments
 */

import {
	loadStripe,
	type Stripe,
	type StripeElements,
	type StripePaymentElement,
	type StripeError,
	type PaymentIntent
} from '@stripe/stripe-js';

// Stripe publishable key from environment
// This is mapped from STRIPE_PUBLISHABLE_KEY in vite.config.ts
const STRIPE_PUBLISHABLE_KEY = import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY || '';

// Import the ApiError from apiAuth
import { ApiError } from './apiAuth';

interface CreatePaymentIntentRequest {
	amount_cents: number;
	currency: string;
}

interface CreatePaymentIntentResponse {
	client_secret: string;
	payment_intent_id: string;
}

interface PaymentStatusResponse {
	has_active_payment: boolean;
	payment_status?: string;
	payment_type?: string;
	subscription_end_date?: string;
}

export class PaymentService {
	private stripe: Stripe | null = null;
	private elements: StripeElements | null = null;

	/**
	 * Initialize Stripe
	 */
	async init(): Promise<void> {
		if (!this.stripe) {
			console.log('PaymentService init - STRIPE_PUBLISHABLE_KEY:', STRIPE_PUBLISHABLE_KEY);
			if (!STRIPE_PUBLISHABLE_KEY) {
				throw new Error(
					'Stripe publishable key not configured. Please set VITE_STRIPE_PUBLISHABLE_KEY in your environment.'
				);
			}

			this.stripe = await loadStripe(STRIPE_PUBLISHABLE_KEY);
			if (!this.stripe) {
				throw new Error('Failed to load Stripe');
			}
			console.log('Stripe loaded successfully');
		}
	}

	/**
	 * Create a payment intent on the server
	 */
	async createPaymentIntent(
		amount_cents: number,
		currency = 'usd'
	): Promise<CreatePaymentIntentResponse> {
		try {
			return await this.apiRequest<CreatePaymentIntentResponse>('/api/payment/create-intent', {
				method: 'POST',
				body: JSON.stringify({
					amount_cents,
					currency
				} satisfies CreatePaymentIntentRequest)
			});
		} catch (error) {
			throw new Error(
				`Payment intent creation failed: ${
					error instanceof ApiError ? error.message : 'Unknown error'
				}`
			);
		}
	}

	/**
	 * Get user payment status
	 */
	async getPaymentStatus(): Promise<PaymentStatusResponse> {
		try {
			return await this.apiRequest<PaymentStatusResponse>('/api/payment/status', {
				method: 'GET'
			});
		} catch (error) {
			throw new Error(
				`Failed to get payment status: ${
					error instanceof ApiError ? error.message : 'Unknown error'
				}`
			);
		}
	}

	/**
	 * Create Stripe elements for payment form
	 */
	createElements(clientSecret: string): StripeElements {
		if (!this.stripe) {
			throw new Error('Stripe not initialized');
		}

		this.elements = this.stripe.elements({
			clientSecret,
			appearance: {
				theme: 'stripe'
			}
		});

		return this.elements;
	}

	/**
	 * Create payment element
	 */
	createPaymentElement(elements: StripeElements): StripePaymentElement {
		const paymentElement = elements.create('payment', {
			// Use default layout which should work better
			layout: {
				type: 'tabs',
				defaultCollapsed: false,
				radios: true,
				spacedAccordionItems: false
			}
		});
		return paymentElement;
	}

	/**
	 * Confirm payment
	 */
	async confirmPayment(
		elements: StripeElements,
		returnUrl: string
	): Promise<{
		error?: StripeError;
		paymentIntent?: PaymentIntent;
	}> {
		if (!this.stripe) {
			throw new Error('Stripe not initialized');
		}

		return this.stripe.confirmPayment({
			elements,
			confirmParams: {
				return_url: returnUrl
			}
		});
	}

	/**
	 * Retrieve payment intent
	 */
	async retrievePaymentIntent(clientSecret: string): Promise<{
		paymentIntent?: PaymentIntent;
		error?: StripeError;
	}> {
		if (!this.stripe) {
			throw new Error('Stripe not initialized');
		}

		return this.stripe.retrievePaymentIntent(clientSecret);
	}

	/**
	 * Get the Stripe instance
	 */
	getStripe(): Stripe {
		if (!this.stripe) {
			throw new Error('Stripe not initialized');
		}
		return this.stripe;
	}

	/**
	 * Internal API request helper with auth
	 */
	private async apiRequest<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
		const API_BASE_URL = `${window.location.protocol}//${window.location.hostname}:${
			import.meta.env.VITE_SERVER_PORT || window.location.port || '8081'
		}`;

		const url = `${API_BASE_URL}${endpoint}`;

		// Default headers
		const headers: Record<string, string> = {
			'Content-Type': 'application/json'
		};

		// Add any existing headers from options
		if (options.headers) {
			Object.assign(headers, options.headers);
		}

		// Add Authorization header if available
		// Use the same key as authStore uses: 'auth_token'
		const token = localStorage.getItem('auth_token');
		if (token) {
			headers.Authorization = `Bearer ${token}`;
		}

		const response = await fetch(url, {
			...options,
			headers
		});

		if (!response.ok) {
			let errorData: { error?: string };
			try {
				errorData = await response.json();
			} catch {
				errorData = { error: `HTTP ${response.status}: ${response.statusText}` };
			}

			throw new ApiError(
				errorData.error || `Request failed with status ${response.status}`,
				response.status
			);
		}

		return response.json();
	}
}

export const paymentService = new PaymentService();
