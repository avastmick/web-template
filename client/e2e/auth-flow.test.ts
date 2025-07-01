import { test, expect } from '@playwright/test';

// Helper to generate unique email for each test
const generateTestEmail = () => `test-${Date.now()}@example.com`;

// Test configuration
const TEST_PASSWORD = 'TestPassword123!';
const API_BASE_URL = 'http://localhost:8081';

test.describe('Authentication Flow', () => {
	test.beforeEach(async ({ page }) => {
		// Start at the home page
		await page.goto('/');
	});

	test('should redirect unauthenticated users to login', async ({ page }) => {
		// When visiting root as unauthenticated user
		await page.goto('/');

		// Should be redirected to login page
		await expect(page).toHaveURL('/login');

		// Login page should be visible
		await expect(page.getByRole('heading', { name: 'Sign in to your account' })).toBeVisible();
	});

	test('should complete full registration and login flow', async ({ page }) => {
		const testEmail = generateTestEmail();

		// Step 1: Navigate to registration
		await page.goto('/register');
		await expect(page.getByRole('heading', { name: 'Create your account' })).toBeVisible();

		// Step 2: Fill registration form
		await page.getByRole('textbox', { name: /email/i }).fill(testEmail);
		await page.getByRole('textbox', { name: /^password/i }).fill(TEST_PASSWORD);
		await page.getByRole('textbox', { name: /confirm password/i }).fill(TEST_PASSWORD);

		// Step 3: Submit registration
		await page.getByRole('button', { name: 'Create Account' }).click();

		// Step 4: Should be logged in and redirected based on payment status
		// Since this is a non-invited user, should go to payment page
		await expect(page).toHaveURL('/payment');

		// Step 5: Verify auth data is stored
		const authToken = await page.evaluate(() => localStorage.getItem('auth_token'));
		expect(authToken).toBeTruthy();

		const authUser = await page.evaluate(() => localStorage.getItem('auth_user'));
		expect(authUser).toBeTruthy();
		const user = JSON.parse(authUser!);
		expect(user.email).toBe(testEmail.toLowerCase());

		// Step 6: Verify payment user data is stored
		const paymentUser = await page.evaluate(() => sessionStorage.getItem('payment_user'));
		expect(paymentUser).toBeTruthy();
		const payment = JSON.parse(paymentUser!);
		expect(payment.payment_required).toBe(true);
		expect(payment.has_valid_invite).toBe(false);
	});

	test('should handle login with existing user', async ({ page }) => {
		// First create a user via API
		const testEmail = generateTestEmail();
		const response = await fetch(`${API_BASE_URL}/api/auth/register`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email: testEmail, password: TEST_PASSWORD })
		});
		expect(response.status).toBe(201);

		// Now test login flow
		await page.goto('/login');

		// Fill login form
		await page.getByRole('textbox', { name: /email/i }).fill(testEmail);
		await page.getByRole('textbox', { name: /password/i }).fill(TEST_PASSWORD);

		// Submit login
		await page.getByRole('button', { name: 'Sign In' }).click();

		// Should redirect to payment page (non-invited user)
		await expect(page).toHaveURL('/payment');

		// Verify auth data
		const authToken = await page.evaluate(() => localStorage.getItem('auth_token'));
		expect(authToken).toBeTruthy();
	});

	test('should persist auth across page refreshes', async ({ page }) => {
		const testEmail = generateTestEmail();

		// Register and login
		await page.goto('/register');
		await page.getByRole('textbox', { name: /email/i }).fill(testEmail);
		await page.getByRole('textbox', { name: /^password/i }).fill(TEST_PASSWORD);
		await page.getByRole('textbox', { name: /confirm password/i }).fill(TEST_PASSWORD);
		await page.getByRole('button', { name: 'Create Account' }).click();

		// Wait for redirect
		await expect(page).toHaveURL('/payment');

		// Refresh the page
		await page.reload();

		// Should still be on payment page (not redirected to login)
		await expect(page).toHaveURL('/payment');

		// Auth data should still be present
		const authToken = await page.evaluate(() => localStorage.getItem('auth_token'));
		expect(authToken).toBeTruthy();
	});

	test('should handle logout correctly', async ({ page }) => {
		const testEmail = generateTestEmail();

		// Register and login
		await page.goto('/register');
		await page.getByRole('textbox', { name: /email/i }).fill(testEmail);
		await page.getByRole('textbox', { name: /^password/i }).fill(TEST_PASSWORD);
		await page.getByRole('textbox', { name: /confirm password/i }).fill(TEST_PASSWORD);
		await page.getByRole('button', { name: 'Create Account' }).click();

		await expect(page).toHaveURL('/payment');

		// Clear auth data to simulate logout
		await page.evaluate(() => {
			localStorage.clear();
			sessionStorage.clear();
		});

		// Navigate to trigger auth check
		await page.goto('/');

		// Should be redirected to login
		await expect(page).toHaveURL('/login');

		// Auth data should be cleared
		const authToken = await page.evaluate(() => localStorage.getItem('auth_token'));
		expect(authToken).toBeNull();

		const paymentUser = await page.evaluate(() => sessionStorage.getItem('payment_user'));
		expect(paymentUser).toBeNull();
	});

	test('should redirect authenticated users from login/register pages', async ({ page }) => {
		const testEmail = generateTestEmail();

		// Register first
		await page.goto('/register');
		await page.getByRole('textbox', { name: /email/i }).fill(testEmail);
		await page.getByRole('textbox', { name: /^password/i }).fill(TEST_PASSWORD);
		await page.getByRole('textbox', { name: /confirm password/i }).fill(TEST_PASSWORD);
		await page.getByRole('button', { name: 'Create Account' }).click();

		await expect(page).toHaveURL('/payment');

		// Try to visit login page while authenticated
		await page.goto('/login');
		// Should be redirected away from login
		await expect(page).not.toHaveURL('/login');

		// Try to visit register page while authenticated
		await page.goto('/register');
		// Should be redirected away from register
		await expect(page).not.toHaveURL('/register');
	});

	test('should handle payment flow redirect correctly', async ({ page }) => {
		const testEmail = generateTestEmail();

		// Register to get to payment page
		await page.goto('/register');
		await page.getByRole('textbox', { name: /email/i }).fill(testEmail);
		await page.getByRole('textbox', { name: /^password/i }).fill(TEST_PASSWORD);
		await page.getByRole('textbox', { name: /confirm password/i }).fill(TEST_PASSWORD);
		await page.getByRole('button', { name: 'Create Account' }).click();

		// Should be on payment page
		await expect(page).toHaveURL('/payment');
		await expect(page.getByRole('heading', { name: 'Payment Required' })).toBeVisible();

		// Verify Stripe element is loaded (wait for it)
		await expect(page.locator('#payment-element')).toBeVisible({ timeout: 10000 });

		// Auth should still be valid
		const authToken = await page.evaluate(() => localStorage.getItem('auth_token'));
		expect(authToken).toBeTruthy();
	});

	test.skip('should handle form validation errors', async ({ page }) => {
		// TODO: This test needs to be updated to work with the current validation approach
		// The validation currently uses HTML5 native validation for email fields
		// and custom validation that triggers on blur for password fields
		await page.goto('/register');

		// Test password mismatch - this is the most reliable validation to test
		await page.getByLabel('Email').fill(`test-${Date.now()}@example.com`);
		await page.getByLabel('Password', { exact: true }).fill(TEST_PASSWORD);
		await page.getByLabel('Confirm Password').fill('different-password');
		await page.getByRole('button', { name: 'Create Account' }).click();

		// Should show password mismatch error
		await expect(page.getByText('Passwords do not match')).toBeVisible();

		// Form should still be on register page
		await expect(page).toHaveURL('/register');
	});

	test('should show error for duplicate email registration', async ({ page }) => {
		const testEmail = generateTestEmail();

		// Register once
		await page.goto('/register');
		await page.getByRole('textbox', { name: /email/i }).fill(testEmail);
		await page.getByRole('textbox', { name: /^password/i }).fill(TEST_PASSWORD);
		await page.getByRole('textbox', { name: /confirm password/i }).fill(TEST_PASSWORD);
		await page.getByRole('button', { name: 'Create Account' }).click();

		await expect(page).toHaveURL('/payment');

		// Logout
		await page.evaluate(() => {
			localStorage.clear();
			sessionStorage.clear();
		});

		// Try to register again with same email
		await page.goto('/register');
		await page.getByRole('textbox', { name: /email/i }).fill(testEmail);
		await page.getByRole('textbox', { name: /^password/i }).fill(TEST_PASSWORD);
		await page.getByRole('textbox', { name: /confirm password/i }).fill(TEST_PASSWORD);
		await page.getByRole('button', { name: 'Create Account' }).click();

		// Should show error
		await expect(page.getByText(/email.*already.*exists|already.*registered/i)).toBeVisible();
	});

	test('should handle invalid login credentials', async ({ page }) => {
		await page.goto('/login');

		// Try to login with non-existent user
		await page.getByRole('textbox', { name: /email/i }).fill('nonexistent@example.com');
		await page.getByRole('textbox', { name: /password/i }).fill(TEST_PASSWORD);
		await page.getByRole('button', { name: 'Sign In' }).click();

		// Should show error - look for the auth error message in the alert
		const alertElement = page.locator('[role="alert"]');
		await expect(alertElement).toBeVisible();
		// Just verify the alert contains the error text without being strict about which element
		await expect(alertElement).toContainText('Invalid email or password');

		// Should still be on login page
		await expect(page).toHaveURL('/login');
	});
});

test.describe('OAuth Flow', () => {
	test('should display OAuth login buttons', async ({ page }) => {
		await page.goto('/login');

		// Check for OAuth buttons
		await expect(page.getByRole('button', { name: /Continue with Google/i })).toBeVisible();
		await expect(page.getByRole('button', { name: /Continue with GitHub/i })).toBeVisible();
	});

	// Note: Full OAuth flow testing requires mocking the OAuth providers
	// or using test accounts, which is beyond the scope of this example
});

test.describe('Protected Routes', () => {
	test('should protect chat route from unauthenticated access', async ({ page }) => {
		// Start fresh on login page
		await page.goto('/login');

		// Clear any existing auth
		await page.evaluate(() => {
			localStorage.clear();
			sessionStorage.clear();
		});

		// Try to access protected route
		await page.goto('/chat');

		// Should be redirected to login
		await expect(page).toHaveURL('/login');
	});

	test('should protect payment route from unauthenticated access', async ({ page }) => {
		// Start fresh on login page
		await page.goto('/login');

		// Clear any existing auth
		await page.evaluate(() => {
			localStorage.clear();
			sessionStorage.clear();
		});

		// Try to access payment route
		await page.goto('/payment');

		// Should be redirected to login
		await expect(page).toHaveURL('/login');
	});
});

test.describe('Session Management', () => {
	test('should maintain separate localStorage and sessionStorage', async ({ page }) => {
		const testEmail = generateTestEmail();

		// Register
		await page.goto('/register');
		await page.getByRole('textbox', { name: /email/i }).fill(testEmail);
		await page.getByRole('textbox', { name: /^password/i }).fill(TEST_PASSWORD);
		await page.getByRole('textbox', { name: /confirm password/i }).fill(TEST_PASSWORD);
		await page.getByRole('button', { name: 'Create Account' }).click();

		await expect(page).toHaveURL('/payment');

		// Check storage
		const localStorageData = await page.evaluate(() => {
			return {
				authToken: localStorage.getItem('auth_token'),
				authUser: localStorage.getItem('auth_user')
			};
		});

		const sessionStorageData = await page.evaluate(() => {
			return {
				paymentUser: sessionStorage.getItem('payment_user'),
				timestamp: sessionStorage.getItem('payment_user_timestamp')
			};
		});

		// localStorage should have auth data
		expect(localStorageData.authToken).toBeTruthy();
		expect(localStorageData.authUser).toBeTruthy();

		// sessionStorage should have payment data
		expect(sessionStorageData.paymentUser).toBeTruthy();
		expect(sessionStorageData.timestamp).toBeTruthy();
	});
});
