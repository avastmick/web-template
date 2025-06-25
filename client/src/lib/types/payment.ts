// web-template/client/src/lib/types/payment.ts

/**
 * Payment-related type definitions
 */

export interface CreatePaymentIntentRequest {
	amount_cents: number;
	currency: string;
}

export interface CreatePaymentIntentResponse {
	client_secret: string;
	payment_intent_id: string;
}

export interface PaymentStatusResponse {
	has_active_payment: boolean;
	payment_status?: 'pending' | 'active' | 'cancelled' | 'expired' | 'failed';
	payment_type?: 'subscription' | 'one_time';
	subscription_end_date?: string;
}

export interface PaymentFormData {
	amount: number;
	currency: string;
	description?: string;
}

export interface PaymentResult {
	success: boolean;
	error?: string;
	paymentIntent?: unknown;
}
