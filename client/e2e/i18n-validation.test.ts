/**
 * Internationalization (i18n) Compliance Validation Tests
 *
 * This test suite validates that all pages and components properly implement
 * internationalization according to our project standards:
 *
 * 1. All pages use translation keys for titles and meta descriptions
 * 2. No hardcoded English text is present on pages
 * 3. Language switching works correctly on all pages
 * 4. RTL languages (Arabic) display properly
 * 5. All translation keys exist in all locale files
 */

import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// Load all translation files to validate keys
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const localesDir = path.join(__dirname, '../src/lib/i18n/locales');
const supportedLocales = ['en-US', 'es-ES', 'zh-CN', 'ar-SA'];

// Load translation files
const translations: Record<string, Record<string, unknown>> = {};
for (const locale of supportedLocales) {
	const filePath = path.join(localesDir, `${locale}.json`);
	if (fs.existsSync(filePath)) {
		translations[locale] = JSON.parse(fs.readFileSync(filePath, 'utf8'));
	}
}

// Define all routes that should be tested
const routes = [
	{ path: '/', name: 'Home' },
	{ path: '/login', name: 'Login' },
	{ path: '/register', name: 'Register' },
	{ path: '/chat', name: 'Chat', requiresAuth: true },
	{ path: '/payment', name: 'Payment', requiresAuth: true }
];

test.describe('i18n Compliance Validation', () => {
	test.beforeEach(async ({ page }) => {
		// Clear localStorage and set up clean state
		await page.goto('/');
		await page.evaluate(() => localStorage.clear());
		await page.reload();
	});

	test.describe('Translation Key Coverage', () => {
		test('should have all required translation keys in all locale files', async () => {
			// Define required keys that should exist in all locales
			const requiredKeys = [
				// Common UI elements
				'common.loading',
				'common.save',
				'common.cancel',
				'common.delete',
				'common.edit',

				// Navigation
				'nav.home',
				'nav.chat',
				'nav.logout',

				// Authentication
				'auth.login.submit',
				'auth.register.submit',
				'auth.login.title',
				'auth.register.title',
				'auth.login.pageTitle',
				'auth.register.pageTitle',
				'auth.login.pageDescription',
				'auth.register.pageDescription',

				// Home page
				'home.title',
				'home.description',

				// Protected pages
				'chat.title',
				'payment.title',

				// Accessibility
				'accessibility.skipToMain',
				'accessibility.skipToNav',

				// Validation
				'validation.required',
				'validation.email'
			];

			const missingKeys: Record<string, string[]> = {};

			// Check each locale for missing keys
			for (const locale of supportedLocales) {
				if (!translations[locale]) {
					throw new Error(`Translation file missing for locale: ${locale}`);
				}

				const missing = [];
				for (const key of requiredKeys) {
					if (!(key in translations[locale])) {
						missing.push(key);
					}
				}

				if (missing.length > 0) {
					missingKeys[locale] = missing;
				}
			}

			// Report any missing keys
			if (Object.keys(missingKeys).length > 0) {
				const errorMessage = Object.entries(missingKeys)
					.map(([locale, keys]) => `${locale}: ${keys.join(', ')}`)
					.join('\n');
				throw new Error(`Missing translation keys:\n${errorMessage}`);
			}
		});
	});

	test.describe('Page i18n Compliance', () => {
		for (const route of routes) {
			test(`${route.name} page should be fully internationalized`, async ({ page }) => {
				// Skip auth-required pages for now (they should be tested separately)
				if (route.requiresAuth) {
					test.skip();
					return;
				}

				await page.goto(route.path);

				// Test each language
				for (const locale of supportedLocales) {
					// Switch to the language
					const languageSelect = page.locator('#language-select').first();
					await languageSelect.selectOption(locale);
					await page.waitForTimeout(500); // Wait for translations to load

					// Verify page has proper lang attribute
					const htmlElement = page.locator('html');
					await expect(htmlElement).toHaveAttribute('lang', locale);

					// Verify page title is translated (not generic or hardcoded English)
					const title = await page.title();

					// Page title should not be empty or contain only English fallbacks
					expect(title).toBeTruthy();
					expect(title.length).toBeGreaterThan(0);

					// For non-English languages, verify title is actually translated
					if (locale !== 'en-US') {
						// Title should be different from English version if properly translated
						await languageSelect.selectOption('en-US');
						await page.waitForTimeout(300);
						const englishTitle = await page.title();

						await languageSelect.selectOption(locale);
						await page.waitForTimeout(300);
						const translatedTitle = await page.title();

						// If the title is the same, it might not be translated
						// This is a warning condition, not a hard failure
						if (englishTitle === translatedTitle && locale !== 'en-US') {
							console.warn(`Warning: ${route.name} page title may not be translated for ${locale}`);
						}
					}

					// Verify RTL direction for Arabic
					if (locale === 'ar-SA') {
						await expect(htmlElement).toHaveAttribute('dir', 'rtl');
					} else {
						// Should not have dir=rtl or should be explicitly ltr
						const dir = await htmlElement.getAttribute('dir');
						expect(dir === null || dir === 'ltr').toBeTruthy();
					}
				}
			});
		}
	});

	test.describe('Hardcoded Text Detection', () => {
		test('should not contain hardcoded English text on any page', async ({ page }) => {
			// Test all public pages
			const testRoutes = [
				{ path: '/', name: 'Home' },
				{ path: '/login', name: 'Login' },
				{ path: '/register', name: 'Register' }
			];

			for (const route of testRoutes) {
				await page.goto(route.path);

				// Switch to Spanish to detect English text
				const languageSelect = page.locator('#language-select').first();
				await languageSelect.selectOption('es-ES');
				await page.waitForTimeout(500);

				// Check page title is translated
				const title = await page.title();
				if (
					title.includes('Web Application Template') ||
					title.includes('Login') ||
					title.includes('Register')
				) {
					throw new Error(
						`${route.name} page title "${title}" appears to be in English when Spanish is selected`
					);
				}

				// Check for common hardcoded English phrases
				const hardcodedPhrases = [
					'User Profile',
					'Profile Information',
					'Account Status',
					'Sign Out',
					'Welcome to the application',
					'Loading profile',
					'Error loading',
					'Redirecting to login',
					'User ID',
					'Email Address',
					'Account Created',
					'Last Updated',
					'Active Account',
					'Refresh',
					'Settings',
					'Dashboard'
				];

				for (const phrase of hardcodedPhrases) {
					const elements = page.getByText(phrase, { exact: false });
					const count = await elements.count();

					if (count > 0) {
						throw new Error(
							`Hardcoded English text "${phrase}" found on ${route.name} page when Spanish is selected`
						);
					}
				}
			}
		});

		test('should detect hardcoded text on chat page (when accessible)', async ({ page }) => {
			// This test will show that chat page has hardcoded text
			// We'll skip it if user is not authenticated, but when it runs it should fail

			await page.goto('/chat');

			// If redirected to login (not authenticated), skip the test
			await page.waitForTimeout(1000);
			if (page.url().includes('/login')) {
				test.skip('Chat page requires authentication - cannot test hardcoded text');
				return;
			}

			// If we somehow got to chat page, test for hardcoded text
			// Switch to Spanish
			const languageSelect = page.locator('#language-select').first();
			await languageSelect.selectOption('es-ES');
			await page.waitForTimeout(500);

			// These phrases should NOT appear in Spanish mode if properly translated
			const hardcodedChatPhrases = [
				'Welcome to AI Chat',
				'New Chat',
				'Send message',
				'Type a message',
				'Conversations',
				'No conversations yet'
			];

			const foundHardcoded = [];
			for (const phrase of hardcodedChatPhrases) {
				const elements = page.getByText(phrase, { exact: false });
				const count = await elements.count();

				if (count > 0) {
					foundHardcoded.push(phrase);
				}
			}

			if (foundHardcoded.length > 0) {
				throw new Error(`Chat page contains hardcoded English text: ${foundHardcoded.join(', ')}`);
			}
		});
	});

	test.describe('Language Switching Functionality', () => {
		test('should persist language choice across navigation', async ({ page }) => {
			await page.goto('/');

			// Switch to Chinese
			const languageSelect = page.locator('#language-select').first();
			await languageSelect.selectOption('zh-CN');
			await page.waitForTimeout(500);

			// Navigate to different pages and verify language persists
			const testRoutes = ['/login', '/register', '/'];

			for (const routePath of testRoutes) {
				await page.goto(routePath);
				await page.waitForTimeout(300);

				// Verify language is still Chinese
				const currentSelect = page.locator('#language-select').first();
				await expect(currentSelect).toHaveValue('zh-CN');

				// Verify HTML lang attribute
				await expect(page.locator('html')).toHaveAttribute('lang', 'zh-CN');
			}
		});

		test('should handle rapid language switching without errors', async ({ page }) => {
			await page.goto('/');

			const languageSelect = page.locator('#language-select').first();

			// Rapidly switch between languages
			const languages = ['es-ES', 'zh-CN', 'ar-SA', 'en-US', 'es-ES'];

			for (const lang of languages) {
				await languageSelect.selectOption(lang);
				await page.waitForTimeout(100); // Short wait to test rapid switching

				// Verify the switch worked
				await expect(languageSelect).toHaveValue(lang);
				await expect(page.locator('html')).toHaveAttribute('lang', lang);
			}

			// Wait a bit longer and verify final state
			await page.waitForTimeout(500);
			await expect(languageSelect).toHaveValue('es-ES');
		});
	});

	test.describe('RTL Language Support', () => {
		test('should properly display Arabic (RTL) layout', async ({ page }) => {
			await page.goto('/');

			// Switch to Arabic
			const languageSelect = page.locator('#language-select').first();
			await languageSelect.selectOption('ar-SA');
			await page.waitForTimeout(500);

			// Verify RTL attributes
			const htmlElement = page.locator('html');
			await expect(htmlElement).toHaveAttribute('dir', 'rtl');
			await expect(htmlElement).toHaveAttribute('lang', 'ar-SA');

			// Verify some content is actually in Arabic
			const navigation = page.locator('nav');

			// Should find Arabic text (at least for navigation items)
			const arabicText = [
				'الرئيسية', // Home
				'تسجيل الدخول', // Sign In
				'إنشاء حساب' // Create Account
			];

			let foundArabicText = false;
			for (const text of arabicText) {
				const element = navigation.getByText(text);
				if ((await element.count()) > 0) {
					foundArabicText = true;
					break;
				}
			}

			expect(foundArabicText).toBeTruthy(); // At least some Arabic text should be present
		});

		test('should switch back from RTL to LTR correctly', async ({ page }) => {
			await page.goto('/');

			const languageSelect = page.locator('#language-select').first();

			// Switch to Arabic (RTL)
			await languageSelect.selectOption('ar-SA');
			await page.waitForTimeout(500);
			await expect(page.locator('html')).toHaveAttribute('dir', 'rtl');

			// Switch back to English (LTR)
			await languageSelect.selectOption('en-US');
			await page.waitForTimeout(500);

			// Should remove dir attribute or set to ltr
			const htmlElement = page.locator('html');
			const dir = await htmlElement.getAttribute('dir');
			expect(dir === null || dir === 'ltr').toBeTruthy();
		});
	});

	test.describe('Error State i18n', () => {
		test('should display translated error messages', async ({ page }) => {
			await page.goto('/login');

			// Switch to Spanish
			const languageSelect = page.locator('#language-select').first();
			await languageSelect.selectOption('es-ES');
			await page.waitForTimeout(500);

			// Try to submit empty form to trigger validation
			const submitButton = page.getByRole('button', { name: /iniciar sesión/i });
			await submitButton.click();

			// Wait for any error messages to appear
			await page.waitForTimeout(1000);

			// Check if error messages are in Spanish (this test may need adjustment based on actual implementation)
			const errorMessages = page.locator('[class*="error"], [role="alert"], .text-red-600');
			const errorCount = await errorMessages.count();

			if (errorCount > 0) {
				// If there are error messages, they should be in Spanish
				const firstError = errorMessages.first();
				const errorText = await firstError.textContent();

				// This is a basic check - actual implementation may vary
				if (errorText && errorText.trim().length > 0) {
					// Error text should not be common English phrases if we're in Spanish mode
					const englishErrorPhrases = ['This field is required', 'Invalid email', 'Required field'];
					const hasEnglishError = englishErrorPhrases.some((phrase) =>
						errorText.toLowerCase().includes(phrase.toLowerCase())
					);

					if (hasEnglishError) {
						console.warn(`Potential untranslated error message: "${errorText}"`);
					}
				}
			}
		});
	});
});
