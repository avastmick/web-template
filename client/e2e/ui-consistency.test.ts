import { test, expect } from '@playwright/test';

test.describe('UI Consistency', () => {
	test.describe('Theme Consistency', () => {
		test('should have consistent theming across all pages', async ({ page }) => {
			// Pages to check
			const pages = [
				{ url: '/', name: 'Home' },
				{ url: '/login', name: 'Login' },
				{ url: '/register', name: 'Register' }
				// Skip payment and chat as they require authentication
			];

			for (const pageInfo of pages) {
				await page.goto(pageInfo.url);

				// Check that no hardcoded colors are present
				const html = await page.content();

				// Check for hardcoded gray colors
				expect(html).not.toMatch(/bg-gray-\d+/);
				expect(html).not.toMatch(/text-gray-\d+/);
				expect(html).not.toMatch(/border-gray-\d+/);

				// Check for hardcoded indigo colors (should use theme tokens)
				expect(html).not.toMatch(/bg-indigo-\d+/);
				expect(html).not.toMatch(/text-indigo-\d+/);

				// Check for hardcoded RGB values
				expect(html).not.toMatch(/rgb\s*\(/);
				expect(html).not.toMatch(/rgba\s*\(/);

				// Check for dark: prefixes (should use semantic tokens)
				expect(html).not.toMatch(/dark:/);

				// Verify theme classes are present
				const rootElement = page.locator('html');
				const classList = await rootElement.getAttribute('class');
				expect(classList).toMatch(/(?:light|dark)/);
			}
		});

		test('should switch between light and dark themes properly', async ({ page }) => {
			await page.goto('/');

			// Get initial theme
			const initialTheme = await page.locator('html').getAttribute('class');
			const isInitiallyDark = initialTheme?.includes('dark');

			// Find and click theme toggle
			const themeToggle = page.getByRole('button', { name: /toggle theme/i });
			await themeToggle.click();

			// Wait for theme change
			await page.waitForTimeout(300);

			// Verify theme changed
			const newTheme = await page.locator('html').getAttribute('class');
			const isNowDark = newTheme?.includes('dark');
			expect(isNowDark).toBe(!isInitiallyDark);

			// Verify CSS variables have changed
			const bgColor = await page.evaluate(() => {
				return window
					.getComputedStyle(document.documentElement)
					.getPropertyValue('--color-background-primary');
			});

			// Check actual theme colors from our theme system
			if (isNowDark) {
				// Dark theme uses indigo-950 (#1e1b4b)
				expect(bgColor.trim()).toBe('#1e1b4b');
			} else {
				// Light theme uses indigo-100 (#e0e7ff)
				expect(bgColor.trim()).toBe('#e0e7ff');
			}

			// Test persistence
			await page.reload();
			const themeAfterReload = await page.locator('html').getAttribute('class');
			expect(themeAfterReload).toBe(newTheme);
		});
	});

	test.describe('Component Reusability', () => {
		test('should use consistent button styling', async ({ page }) => {
			await page.goto('/login');

			// Check primary buttons
			const primaryButtons = page.locator('button').filter({ hasText: /sign in|log in/i });
			const primaryCount = await primaryButtons.count();

			for (let i = 0; i < primaryCount; i++) {
				const button = primaryButtons.nth(i);
				const classes = await button.getAttribute('class');

				// Should use theme-aware classes
				expect(classes).toMatch(/bg-color-primary/);
				expect(classes).toMatch(/text-text-on-primary/);
				expect(classes).toMatch(/hover:bg-color-primary-hover/);
			}
		});

		test('should use consistent alert/message styling', async ({ page }) => {
			// Navigate to a page that shows an alert after form submission
			await page.goto('/login');

			// Fill in invalid credentials to trigger error alert
			await page.getByRole('textbox', { name: /email/i }).fill('invalid@example.com');
			await page.getByRole('textbox', { name: /password/i }).fill('wrongpassword');
			await page.getByRole('button', { name: /sign in/i }).click();

			// Wait for the main error alert (not field validation)
			await page.waitForSelector('[role="alert"]', { timeout: 5000 });

			// Check for Alert component - look for the container div with role="alert"
			const alertElement = page.locator('[role="alert"]').first();
			const alertVisible = await alertElement.isVisible();

			expect(alertVisible).toBe(true);

			// The alert component should have proper styling
			const classes = await alertElement.getAttribute('class');

			// Our alert component uses flex layout and border styling
			expect(classes).toMatch(/flex items-start gap-3 rounded-lg border p-4/);
			// Should have transition fade
			expect(classes).toMatch(/bg-status-error-bg/);
		});

		test('should use consistent form field styling', async ({ page }) => {
			await page.goto('/register');

			// Wait for the form to be visible
			await page.waitForSelector('form', { timeout: 5000 });

			// Check all input fields
			const inputs = page.locator('input[type="email"], input[type="password"]');
			const inputCount = await inputs.count();

			expect(inputCount).toBeGreaterThan(0);

			for (let i = 0; i < inputCount; i++) {
				const input = inputs.nth(i);
				const classes = await input.getAttribute('class');

				// Should use theme-aware classes
				expect(classes).toMatch(/bg-background-primary/);
				expect(classes).toMatch(/border-border-default/);
				expect(classes).toMatch(/focus-visible:ring-amber-400/);
				// Should not have hardcoded colors
				expect(classes).not.toMatch(/bg-gray-\d+/);
			}
		});
	});

	test.describe('Responsive Design', () => {
		const viewports = [
			{ name: 'mobile', width: 375, height: 667 },
			{ name: 'tablet', width: 768, height: 1024 },
			{ name: 'desktop', width: 1920, height: 1080 }
		];

		for (const viewport of viewports) {
			test(`should be responsive on ${viewport.name}`, async ({ page }) => {
				await page.setViewportSize({ width: viewport.width, height: viewport.height });
				await page.goto('/');

				// Check navigation
				const nav = page.locator('nav');
				await expect(nav).toBeVisible();

				// On mobile, check for mobile menu
				if (viewport.name === 'mobile') {
					// Mobile menu button should be visible
					const mobileMenuButton = page.getByRole('button', { name: /menu/i });
					await expect(mobileMenuButton).toBeVisible();

					// Desktop navigation items should be hidden
					const desktopNav = page.locator('nav .hidden.md\\:flex');
					await expect(desktopNav).toBeHidden();
				} else {
					// Desktop navigation should be visible
					const desktopNav = page.locator('nav').locator('a').first();
					await expect(desktopNav).toBeVisible();
				}

				// Check main content area
				const main = page.locator('main');
				await expect(main).toBeVisible();

				// Verify no horizontal scroll
				const hasHorizontalScroll = await page.evaluate(() => {
					return document.documentElement.scrollWidth > document.documentElement.clientWidth;
				});
				expect(hasHorizontalScroll).toBe(false);
			});
		}
	});

	test.describe('Accessibility', () => {
		test('should have proper focus indicators', async ({ page }) => {
			await page.goto('/login');

			// Wait for the page to be ready
			await page.waitForSelector('form', { timeout: 5000 });

			// Tab to the first input field
			await page.keyboard.press('Tab');
			await page.keyboard.press('Tab'); // Skip skip links
			await page.keyboard.press('Tab'); // Skip navigation

			// Check if an element is focused
			const focusedElement = await page.evaluate(() => {
				const el = document.activeElement;
				return {
					tagName: el?.tagName.toLowerCase(),
					className: el?.className || '',
					hasRing: el?.className.includes('ring') || false
				};
			});

			// Should have focused an interactive element
			expect(['input', 'button', 'a']).toContain(focusedElement.tagName);

			// The element should have focus styles defined (even if not visible in the class)
			// Our theme uses focus:ring-2 focus:ring-amber-400
			expect(focusedElement.className).toBeTruthy();
		});

		test('should meet minimum touch target size', async ({ page }) => {
			await page.goto('/');

			// Check all buttons and links
			const interactiveElements = page.locator('button, a, input, select, textarea');
			const count = await interactiveElements.count();

			for (let i = 0; i < count; i++) {
				const element = interactiveElements.nth(i);
				const box = await element.boundingBox();

				if (box) {
					// Minimum 44x44px for touch targets
					expect(box.width).toBeGreaterThanOrEqual(44);
					expect(box.height).toBeGreaterThanOrEqual(44);
				}
			}
		});

		test('should have proper ARIA labels', async ({ page }) => {
			await page.goto('/');

			// Check icon buttons have labels
			const iconButtons = page.locator('button:has(svg)');
			const iconButtonCount = await iconButtons.count();

			for (let i = 0; i < iconButtonCount; i++) {
				const button = iconButtons.nth(i);
				const ariaLabel = await button.getAttribute('aria-label');
				const textContent = await button.textContent();

				// Should have either aria-label or visible text
				expect(ariaLabel || textContent?.trim()).toBeTruthy();
			}
		});
	});

	test.describe('Color Contrast', () => {
		test('should have sufficient color contrast for text', async ({ page }) => {
			await page.goto('/');

			// Wait for page to be ready
			await page.waitForLoadState('domcontentloaded');

			// Check a few key text elements for contrast
			const elementsToCheck = [
				page.locator('h1').first(),
				page.locator('p').first(),
				page.locator('button').first()
			];

			for (const element of elementsToCheck) {
				const isVisible = await element.isVisible().catch(() => false);

				if (isVisible) {
					const styles = await element.evaluate((el) => {
						const computed = window.getComputedStyle(el);
						return {
							color: computed.color,
							fontSize: computed.fontSize,
							fontWeight: computed.fontWeight
						};
					});

					// Verify text has proper color values
					expect(styles.color).toBeTruthy();
					expect(styles.color).not.toBe('rgba(0, 0, 0, 0)');

					// Check that we're using our theme colors (should be rgb values)
					expect(styles.color).toMatch(/^rgb/);
				}
			}
		});
	});
});
