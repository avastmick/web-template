// web-template/client/src/lib/services/storageService.ts

/**
 * Storage Service for managing localStorage and sessionStorage
 *
 * This service provides a centralized way to manage auth data persistence:
 * - Auth token and user data in localStorage (persistent across sessions)
 * - Payment status in sessionStorage (session-only, cleared on browser close)
 */

import { browser } from '$app/environment';
import type { User, PaymentUser } from '$lib/types/auth';

// Storage keys
const AUTH_TOKEN_KEY = 'auth_token';
const AUTH_USER_KEY = 'auth_user';
const PAYMENT_USER_KEY = 'payment_user';
const PAYMENT_USER_TIMESTAMP_KEY = 'payment_user_timestamp';

// Cache duration for payment status (5 minutes)
const PAYMENT_CACHE_DURATION = 5 * 60 * 1000;

export class StorageService {
	/**
	 * Store auth token in localStorage
	 */
	static setAuthToken(token: string): void {
		if (!browser) return;
		try {
			localStorage.setItem(AUTH_TOKEN_KEY, token);
		} catch (error) {
			console.error('[StorageService] Failed to store auth token:', error);
		}
	}

	/**
	 * Get auth token from localStorage
	 */
	static getAuthToken(): string | null {
		if (!browser) return null;
		try {
			return localStorage.getItem(AUTH_TOKEN_KEY);
		} catch (error) {
			console.error('[StorageService] Failed to get auth token:', error);
			return null;
		}
	}

	/**
	 * Store auth user in localStorage
	 */
	static setAuthUser(user: User): void {
		if (!browser) return;
		try {
			localStorage.setItem(AUTH_USER_KEY, JSON.stringify(user));
		} catch (error) {
			console.error('[StorageService] Failed to store auth user:', error);
		}
	}

	/**
	 * Get auth user from localStorage
	 */
	static getAuthUser(): User | null {
		if (!browser) return null;
		try {
			const userStr = localStorage.getItem(AUTH_USER_KEY);
			return userStr ? JSON.parse(userStr) : null;
		} catch (error) {
			console.error('[StorageService] Failed to get auth user:', error);
			return null;
		}
	}

	/**
	 * Store payment user in sessionStorage with timestamp
	 */
	static setPaymentUser(paymentUser: PaymentUser): void {
		if (!browser) return;
		try {
			sessionStorage.setItem(PAYMENT_USER_KEY, JSON.stringify(paymentUser));
			sessionStorage.setItem(PAYMENT_USER_TIMESTAMP_KEY, Date.now().toString());
		} catch (error) {
			console.error('[StorageService] Failed to store payment user:', error);
		}
	}

	/**
	 * Get payment user from sessionStorage
	 */
	static getPaymentUser(): PaymentUser | null {
		if (!browser) return null;
		try {
			const paymentStr = sessionStorage.getItem(PAYMENT_USER_KEY);
			return paymentStr ? JSON.parse(paymentStr) : null;
		} catch (error) {
			console.error('[StorageService] Failed to get payment user:', error);
			return null;
		}
	}

	/**
	 * Check if payment user data is stale
	 */
	static isPaymentUserStale(): boolean {
		if (!browser) return true;
		try {
			const timestamp = sessionStorage.getItem(PAYMENT_USER_TIMESTAMP_KEY);
			if (!timestamp) return true;

			const age = Date.now() - parseInt(timestamp, 10);
			return age > PAYMENT_CACHE_DURATION;
		} catch (error) {
			console.error('[StorageService] Failed to check payment staleness:', error);
			return true;
		}
	}

	/**
	 * Clear auth data from localStorage
	 */
	static clearAuthData(): void {
		if (!browser) return;
		try {
			localStorage.removeItem(AUTH_TOKEN_KEY);
			localStorage.removeItem(AUTH_USER_KEY);
		} catch (error) {
			console.error('[StorageService] Failed to clear auth data:', error);
		}
	}

	/**
	 * Clear payment data from sessionStorage
	 */
	static clearPaymentData(): void {
		if (!browser) return;
		try {
			sessionStorage.removeItem(PAYMENT_USER_KEY);
			sessionStorage.removeItem(PAYMENT_USER_TIMESTAMP_KEY);
		} catch (error) {
			console.error('[StorageService] Failed to clear payment data:', error);
		}
	}

	/**
	 * Clear all auth-related data
	 */
	static clearAll(): void {
		this.clearAuthData();
		this.clearPaymentData();
	}

	/**
	 * Check if user is authenticated (has token and user data)
	 */
	static isAuthenticated(): boolean {
		return !!this.getAuthToken() && !!this.getAuthUser();
	}

	/**
	 * Get authorization header for API requests
	 */
	static getAuthHeader(): { Authorization?: string } {
		const token = this.getAuthToken();
		return token ? { Authorization: `Bearer ${token}` } : {};
	}

	/**
	 * Check if payment is valid (not expired)
	 */
	static isPaymentValid(paymentUser: PaymentUser | null): boolean {
		if (!paymentUser) return false;
		if (!paymentUser.payment_required) return true;

		// Check invite expiry
		if (paymentUser.has_valid_invite && paymentUser.invite_expires_at) {
			const expiryDate = new Date(paymentUser.invite_expires_at);
			if (expiryDate > new Date()) return true;
		}

		// Check subscription expiry
		if (paymentUser.payment_status === 'active' && paymentUser.subscription_end_date) {
			const endDate = new Date(paymentUser.subscription_end_date);
			if (endDate > new Date()) return true;
		}

		return false;
	}
}
