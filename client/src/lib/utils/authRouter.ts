import { browser } from '$app/environment';
import { goto } from '$app/navigation';

/**
 * Simple auth router that checks auth and payment status
 * and navigates to the appropriate page
 */
export async function navigateBasedOnAuthStatus(targetRoute = '/'): Promise<void> {
	if (!browser) return;

	// Get auth data from localStorage
	const authToken = localStorage.getItem('auth_token');
	const authUser = localStorage.getItem('auth_user');

	if (!authToken || !authUser) {
		// Not authenticated - go to login
		await goto('/login');
		return;
	}

	// If going to root, check payment status to determine destination
	if (targetRoute === '/') {
		// Check payment status from sessionStorage
		const paymentStatusChecked = sessionStorage.getItem('payment_status_checked') === 'true';
		const paymentRequired = sessionStorage.getItem('payment_required') === 'true';

		if (paymentStatusChecked && !paymentRequired) {
			// User has paid or is invited - go to chat
			await goto('/chat');
		} else if (paymentStatusChecked && paymentRequired) {
			// User needs to pay - go to payment
			await goto('/payment');
		} else {
			// Payment status not checked yet - fetch from server
			try {
				const { getCurrentUser } = await import('$lib/services/apiAuth');
				const userData = await getCurrentUser();

				// Store payment status in sessionStorage
				sessionStorage.setItem('payment_status_checked', 'true');
				sessionStorage.setItem(
					'payment_required',
					userData.payment_user.payment_required.toString()
				);

				// Navigate based on payment status
				if (userData.payment_user.payment_required) {
					await goto('/payment');
				} else {
					await goto('/chat');
				}
			} catch (error) {
				console.error('Failed to check payment status:', error);
				// If check fails, go to login
				await goto('/login');
			}
		}
	} else {
		// Navigate to the requested route
		await goto(targetRoute);
	}
}
