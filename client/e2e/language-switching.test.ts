/**
 * Language Switching E2E Tests
 *
 * Tests the internationalization (i18n) functionality including:
 * - Language switching via LanguageSelector
 * - Persistence of language choice
 * - UI updates with correct translations
 * - RTL support for Arabic
 */

import { test, expect } from '@playwright/test';

test.describe('Language Switching', () => {
	test.beforeEach(async ({ page }) => {
		// Clear localStorage before each test
		await page.goto('/');
		await page.evaluate(() => localStorage.clear());
		// Refresh to ensure clean state
		await page.reload();
	});

	test('should default to English language', async ({ page }) => {
		await page.goto('/');

		// Check if page title is in English
		await expect(page).toHaveTitle('Web Application Template');

		// Check if navigation is in English (unauthenticated state)
		await expect(page.locator('nav').getByText('Home')).toBeVisible();
		await expect(page.locator('nav').getByText('Sign In')).toBeVisible();
		await expect(page.locator('nav').getByText('Create Account')).toBeVisible();

		// Check if main content is in English
		await expect(page.getByText('Web Application Template')).toBeVisible();
		await expect(
			page.getByText('A modern, secure web application built with SvelteKit and Rust')
		).toBeVisible();
	});

	test('should switch to Spanish language', async ({ page }) => {
		await page.goto('/');

		// Find and click language selector (use first one - desktop)
		const languageSelect = page.locator('#language-select').first();
		await expect(languageSelect).toBeVisible();

		// Switch to Spanish
		await languageSelect.selectOption('es-ES');

		// Wait for translations to load and page to update
		await page.waitForTimeout(500);

		// Check if content is now in Spanish (unauthenticated state)
		await expect(page.locator('nav').getByText('Inicio')).toBeVisible();
		await expect(page.locator('nav').getByText('Iniciar Sesión')).toBeVisible();
		await expect(page.locator('nav').getByText('Crear Cuenta')).toBeVisible();

		// Check main content in Spanish
		await expect(page.getByText('Plantilla de Aplicación Web')).toBeVisible();
	});

	test('should switch to Chinese language', async ({ page }) => {
		await page.goto('/');

		// Switch to Chinese
		const languageSelect = page.locator('#language-select').first();
		await languageSelect.selectOption('zh-CN');

		// Wait for translations to load
		await page.waitForTimeout(500);

		// Check if content is now in Chinese (unauthenticated state)
		await expect(page.locator('nav').getByText('首页')).toBeVisible();
		await expect(page.locator('nav').getByText('登录')).toBeVisible();
		await expect(page.locator('nav').getByText('创建账户')).toBeVisible();

		// Check main content in Chinese
		await expect(page.getByText('Web应用程序模板')).toBeVisible();
	});

	test('should switch to Arabic and apply RTL direction', async ({ page }) => {
		await page.goto('/');

		// Switch to Arabic
		const languageSelect = page.locator('#language-select').first();
		await languageSelect.selectOption('ar-SA');

		// Wait for translations to load
		await page.waitForTimeout(500);

		// Check if content is now in Arabic (unauthenticated state)
		await expect(page.locator('nav').getByText('الرئيسية')).toBeVisible();
		await expect(page.locator('nav').getByText('تسجيل الدخول')).toBeVisible();
		await expect(page.locator('nav').getByText('إنشاء حساب')).toBeVisible();

		// Check main content in Arabic
		await expect(page.getByText('قالب تطبيق الويب')).toBeVisible();

		// Check if RTL direction is applied
		const htmlElement = page.locator('html');
		await expect(htmlElement).toHaveAttribute('dir', 'rtl');
		await expect(htmlElement).toHaveAttribute('lang', 'ar-SA');
	});

	test('should persist language choice across page reloads', async ({ page }) => {
		await page.goto('/');

		// Switch to Spanish
		const languageSelect = page.locator('#language-select').first();
		await languageSelect.selectOption('es-ES');
		await page.waitForTimeout(500);

		// Verify Spanish is selected
		await expect(languageSelect).toHaveValue('es-ES');
		await expect(page.locator('nav').getByText('Inicio')).toBeVisible();

		// Reload the page
		await page.reload();
		await page.waitForTimeout(500);

		// Check if Spanish is still selected and content is in Spanish
		await expect(languageSelect).toHaveValue('es-ES');
		await expect(page.locator('nav').getByText('Inicio')).toBeVisible();
		await expect(page.getByText('Plantilla de Aplicación Web')).toBeVisible();
	});

	test('should persist language choice across navigation', async ({ page }) => {
		await page.goto('/');

		// Switch to Chinese
		const languageSelect = page.locator('#language-select').first();
		await languageSelect.selectOption('zh-CN');
		await page.waitForTimeout(500);

		// Navigate to login page
		await page.goto('/login');
		await page.waitForTimeout(500);

		// Check if login page is in Chinese
		await expect(page.getByText('登录您的账户')).toBeVisible();
		await expect(page.getByText('欢迎回来！请输入您的详细信息。')).toBeVisible();

		// Check if language selector still shows Chinese
		const loginLanguageSelect = page.locator('#language-select').first();
		await expect(loginLanguageSelect).toHaveValue('zh-CN');
	});

	test('should translate login page correctly', async ({ page }) => {
		await page.goto('/login');

		// Test English (default)
		await expect(page.getByText('Sign in to your account')).toBeVisible();
		await expect(page.getByText('Welcome back! Please enter your details.')).toBeVisible();
		await expect(page.getByText("Don't have an account?")).toBeVisible();

		// Switch to Spanish
		const languageSelect = page.locator('#language-select').first();
		await languageSelect.selectOption('es-ES');
		await page.waitForTimeout(500);

		// Check Spanish translations
		await expect(page.getByText('Inicia sesión en tu cuenta')).toBeVisible();
		await expect(
			page.getByText('¡Bienvenido de nuevo! Por favor, ingresa tus datos.')
		).toBeVisible();
		await expect(page.getByText('¿No tienes una cuenta?')).toBeVisible();

		// Test form labels
		await expect(page.getByLabel('Correo Electrónico')).toBeVisible();
		await expect(page.getByLabel('Contraseña')).toBeVisible();
	});

	test('should translate registration page correctly', async ({ page }) => {
		await page.goto('/register');

		// Test English (default)
		await expect(page.getByText('Create your account')).toBeVisible();
		await expect(page.getByLabel('Email')).toBeVisible();

		// Switch to Spanish
		const languageSelect = page.locator('#language-select').first();
		await languageSelect.selectOption('es-ES');
		await page.waitForTimeout(500);

		// Check Spanish translations
		await expect(page.getByText('Crea tu cuenta')).toBeVisible();
		await expect(page.getByLabel('Correo Electrónico')).toBeVisible();

		// Test form labels and placeholders
		await expect(page.getByLabel('Correo Electrónico')).toBeVisible();
		await expect(page.getByLabel('Contraseña', { exact: true })).toBeVisible();
		await expect(page.getByLabel('Confirmar Contraseña')).toBeVisible();
	});

	test('should translate OAuth buttons correctly', async ({ page }) => {
		await page.goto('/login');

		// Test English (default)
		await expect(page.getByText('Continue with Google')).toBeVisible();
		await expect(page.getByText('Continue with GitHub')).toBeVisible();

		// Switch to Chinese
		const languageSelect = page.locator('#language-select').first();
		await languageSelect.selectOption('zh-CN');
		await page.waitForTimeout(500);

		// Check Chinese translations
		await expect(page.getByText('继续使用Google')).toBeVisible();
		await expect(page.getByText('继续使用GitHub')).toBeVisible();
	});

	test('should handle language switching during form interaction', async ({ page }) => {
		await page.goto('/login');

		// Check English form elements
		await expect(page.getByLabel('Email')).toBeVisible();
		await expect(page.getByLabel('Password')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Sign In' })).toBeVisible();

		// Switch to Spanish
		const languageSelect = page.locator('#language-select').first();
		await languageSelect.selectOption('es-ES');
		await page.waitForTimeout(500);

		// Check Spanish form elements
		await expect(page.getByLabel('Correo Electrónico')).toBeVisible();
		await expect(page.getByLabel('Contraseña')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Iniciar Sesión' })).toBeVisible();
	});

	test('should update page title and meta description based on language', async ({ page }) => {
		// Test login page title
		await page.goto('/login');
		await expect(page).toHaveTitle('Sign In - Login to Your Account');

		// Switch to Spanish
		const languageSelect = page.locator('#language-select').first();
		await languageSelect.selectOption('es-ES');
		await page.waitForTimeout(500);

		// Should update title
		await expect(page).toHaveTitle('Iniciar Sesión - Acceder a tu Cuenta');

		// Test registration page
		await page.goto('/register');
		await page.waitForTimeout(500);
		await expect(page).toHaveTitle('Registro - Crear Cuenta');
	});

	test('should maintain accessibility with language changes', async ({ page }) => {
		await page.goto('/');

		// Check language selector has proper accessibility attributes
		const languageSelect = page.locator('#language-select').first();
		await expect(languageSelect).toHaveAttribute('aria-label', 'Select language');

		// Switch to Arabic (RTL)
		await languageSelect.selectOption('ar-SA');
		await page.waitForTimeout(500);

		// Check if lang attribute is updated
		const htmlElement = page.locator('html');
		await expect(htmlElement).toHaveAttribute('lang', 'ar-SA');
		await expect(htmlElement).toHaveAttribute('dir', 'rtl');

		// Verify language selector is still accessible
		await expect(languageSelect).toHaveAttribute('aria-label', 'Select language');
	});
});
