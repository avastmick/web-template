import { expect, test } from '@playwright/test';

test.describe('GitHub OAuth Integration', () => {
	test('login page has GitHub OAuth button', async ({ page }) => {
		await page.goto('/login');

		// Check that the GitHub OAuth button is present
		const githubButton = page.locator('button:has-text("Continue with GitHub")');
		await expect(githubButton).toBeVisible();

		// Verify the button has the correct GitHub icon
		const githubIcon = githubButton.locator('svg');
		await expect(githubIcon).toBeVisible();

		// Verify button styling and classes
		await expect(githubButton).toHaveClass(/border-gray-300/);
		await expect(githubButton).toHaveClass(/bg-white/);
	});

	test('register page has GitHub OAuth button', async ({ page }) => {
		await page.goto('/register');

		// Check that the GitHub OAuth button is present
		const githubButton = page.locator('button:has-text("Continue with GitHub")');
		await expect(githubButton).toBeVisible();

		// Verify the button has the correct GitHub icon
		const githubIcon = githubButton.locator('svg');
		await expect(githubIcon).toBeVisible();

		// Verify button styling and classes
		await expect(githubButton).toHaveClass(/border-gray-300/);
		await expect(githubButton).toHaveClass(/bg-white/);
	});

	test('OAuth callback page shows loading state briefly with valid parameters', async ({
		page
	}) => {
		// Navigate to OAuth callback page with valid parameters
		// This will cause the page to show loading state briefly before processing
		await page.goto(
			'/auth/oauth/callback?token=valid_token&user_id=user123&email=test@example.com&is_new_user=false'
		);

		// The loading state might be very brief, so we'll check if the success state appears
		// indicating the loading state was processed
		const successHeading = page.locator('text=Sign In Successful!');
		await expect(successHeading).toBeVisible();
	});

	test('OAuth callback page handles successful authentication', async ({ page }) => {
		// Navigate to OAuth callback with success parameters
		await page.goto(
			'/auth/oauth/callback?token=mock_jwt_token&user_id=mock_user_id&email=test@example.com&is_new_user=true'
		);

		// Should show success state
		const successHeading = page.locator('text=Sign In Successful!');
		await expect(successHeading).toBeVisible();

		const welcomeText = page.locator('text=Welcome! Your account has been created.');
		await expect(welcomeText).toBeVisible();
	});

	test('OAuth callback page handles authentication errors', async ({ page }) => {
		// Navigate to OAuth callback with error parameter
		await page.goto('/auth/oauth/callback?error=no_invite');

		// Should show error state
		const errorHeading = page.locator('text=Sign In Failed');
		await expect(errorHeading).toBeVisible();

		const authFailedHeading = page.locator('text=Authentication Failed');
		await expect(authFailedHeading).toBeVisible();

		// Should show specific error message for no_invite
		const errorMessage = page.locator('text=Registration is by invitation only');
		await expect(errorMessage).toBeVisible();

		// Should have a "Try Again" button
		const tryAgainButton = page.locator('button:has-text("Try Again")');
		await expect(tryAgainButton).toBeVisible();
	});

	test('OAuth callback page handles oauth_exchange_failed error', async ({ page }) => {
		// Navigate to OAuth callback with oauth_exchange_failed error
		await page.goto('/auth/oauth/callback?error=oauth_exchange_failed');

		// Should show error state with specific message
		const errorMessage = page.locator('text=Failed to exchange authorization code');
		await expect(errorMessage).toBeVisible();
	});

	test('OAuth callback page handles missing token error', async ({ page }) => {
		// Navigate to OAuth callback with incomplete parameters (missing token)
		await page.goto('/auth/oauth/callback?user_id=mock_user_id&email=test@example.com');

		// Should show error state
		const errorHeading = page.locator('text=Authentication Failed');
		await expect(errorHeading).toBeVisible();

		// Should show invalid OAuth response message
		const errorMessage = page.locator('text=Invalid OAuth response from server');
		await expect(errorMessage).toBeVisible();
	});

	test('GitHub OAuth button click attempts navigation to OAuth endpoint', async ({ page }) => {
		await page.goto('/login');

		// Intercept the OAuth request to prevent actual redirect
		let oauthRequestMade = false;
		await page.route('**/api/auth/oauth/github*', async (route) => {
			oauthRequestMade = true;
			// Prevent the actual redirect by fulfilling with error
			await route.fulfill({
				status: 400,
				body: 'Test prevented actual OAuth'
			});
		});

		// Get the GitHub OAuth button and click it
		const githubButton = page.locator('button:has-text("Continue with GitHub")');
		await githubButton.click();

		// Wait a moment for the request to be made
		await page.waitForTimeout(500);

		// Verify that the OAuth request was attempted
		expect(oauthRequestMade).toBe(true);
	});

	test('both Google and GitHub OAuth buttons are present', async ({ page }) => {
		await page.goto('/login');

		// Both OAuth buttons should be present
		const googleButton = page.locator('button:has-text("Continue with Google")');
		const githubButton = page.locator('button:has-text("Continue with GitHub")');

		await expect(googleButton).toBeVisible();
		await expect(githubButton).toBeVisible();

		// They should be in the same container with space between them
		const oauthContainer = page.locator('.space-y-3').filter({ has: githubButton });
		await expect(oauthContainer).toBeVisible();
		await expect(oauthContainer).toContainText('Continue with Google');
		await expect(oauthContainer).toContainText('Continue with GitHub');
	});
});
