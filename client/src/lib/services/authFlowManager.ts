// web-template/client/src/lib/services/authFlowManager.ts

/**
 * Auth Flow Manager - Centralized auth flow control
 *
 * This service manages authentication flow and navigation:
 * - Uses cached payment status from sessionStorage
 * - Only refreshes payment data when stale
 * - Handles all auth-related redirects
 * - Provides single source of truth for auth state
 */

import { browser } from '$app/environment';
import { get } from 'svelte/store';
import { authStore, paymentUser } from '$lib/stores';
import { StorageService } from './storageService';
import { getCurrentUser } from './apiAuth';
import type { PaymentUser } from '$lib/types/auth';

// Configurable success route
const SUCCESS_ROUTE = '/chat';

export class AuthFlowManager {
	/**
	 * Check if user needs to pay
	 */
	static async needsPayment(): Promise<boolean> {
		if (!browser) return false;

		// Get cached payment user data
		let paymentUserData: PaymentUser | null = get(paymentUser);

		// If no data in store, try sessionStorage
		if (!paymentUserData) {
			paymentUserData = StorageService.getPaymentUser();
			if (paymentUserData) {
				authStore.setPaymentUser(paymentUserData);
			}
		}

		// If data is stale or missing, refresh from server
		if (!paymentUserData || StorageService.isPaymentUserStale()) {
			try {
				const userData = await getCurrentUser();
				// Store is updated by getCurrentUser via handleAuthResponse
				paymentUserData = userData.payment_user;
			} catch (error) {
				console.error('[AuthFlowManager] Failed to refresh payment status:', error);
				// If refresh fails, assume payment is needed for safety
				return true;
			}
		}

		// Check if payment is valid
		return paymentUserData?.payment_required && !StorageService.isPaymentValid(paymentUserData);
	}

	/**
	 * Get the appropriate redirect URL based on auth and payment status
	 */
	static async getRedirectUrl(): Promise<string> {
		if (!StorageService.isAuthenticated()) {
			return '/login';
		}

		const needsPayment = await this.needsPayment();
		return needsPayment ? '/payment' : SUCCESS_ROUTE;
	}

	/**
	 * Handle navigation after successful authentication
	 */
	static async handleAuthSuccess(): Promise<void> {
		if (!browser) return;

		const redirectUrl = await this.getRedirectUrl();
		// Use window.location.href to avoid UI stacking issue
		window.location.href = redirectUrl;
	}

	/**
	 * Handle navigation for public routes (login/register)
	 */
	static async handlePublicRoute(): Promise<void> {
		if (!browser) return;

		// Wait for auth store to initialize
		await authStore.waitForInit();

		if (StorageService.isAuthenticated()) {
			// User is authenticated, redirect to appropriate page
			await this.handleAuthSuccess();
		}
	}

	/**
	 * Handle navigation for protected routes
	 */
	static async handleProtectedRoute(requirePayment = true): Promise<boolean> {
		if (!browser) return true;

		// Wait for auth store to initialize
		await authStore.waitForInit();

		if (!StorageService.isAuthenticated()) {
			// User needs to authenticate
			window.location.href = '/login';
			return false;
		}

		if (requirePayment) {
			const needsPayment = await this.needsPayment();
			if (needsPayment) {
				// User needs to pay
				window.location.href = '/payment';
				return false;
			}
		}

		return true;
	}

	/**
	 * Handle root page redirect
	 */
	static async handleRootRedirect(): Promise<void> {
		if (!browser) return;

		// Wait for auth store to initialize
		await authStore.waitForInit();

		const redirectUrl = await this.getRedirectUrl();
		window.location.href = redirectUrl;
	}

	/**
	 * Handle post-payment success
	 */
	static async handlePaymentSuccess(updatedPaymentUser: PaymentUser): Promise<void> {
		if (!browser) return;

		// Update payment user data
		authStore.setPaymentUser(updatedPaymentUser);

		// Redirect to success route
		window.location.href = SUCCESS_ROUTE;
	}

	/**
	 * Clear all auth data and redirect to login
	 */
	static async logout(): Promise<void> {
		authStore.logout();
		if (browser) {
			window.location.href = '/login';
		}
	}
}
