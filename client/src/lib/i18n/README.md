# Internationalization (i18n) System

This document provides comprehensive guidelines for implementing, maintaining, and extending the internationalization system in this SvelteKit application.

## 📋 Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Translation Key Guidelines](#translation-key-guidelines)
- [Adding New Languages](#adding-new-languages)
- [Component Implementation](#component-implementation)
- [Page Implementation](#page-implementation)
- [Testing & Validation](#testing--validation)
- [RTL Language Support](#rtl-language-support)
- [Troubleshooting](#troubleshooting)

## 🌍 Overview

Our i18n system is built with `svelte-i18n` and supports:

- **4 Languages**: English (en-US), Spanish (es-ES), Chinese (zh-CN), Arabic (ar-SA)
- **RTL Support**: Automatic direction switching for Arabic
- **Lazy Loading**: Translation bundles loaded on-demand
- **Persistence**: Language choice stored in localStorage
- **TypeScript**: Full type safety for translation keys

### Supported Locales

| Locale  | Language                | Direction | Status      |
| ------- | ----------------------- | --------- | ----------- |
| `en-US` | English (United States) | LTR       | ✅ Complete |
| `es-ES` | Spanish (Spain)         | LTR       | ✅ Complete |
| `zh-CN` | Chinese (Simplified)    | LTR       | ✅ Complete |
| `ar-SA` | Arabic (Saudi Arabia)   | RTL       | ✅ Complete |

## 🚀 Quick Start

### 1. Import Translation Function

In every component/page that displays user-facing text:

```typescript
import { _ } from 'svelte-i18n';
```

### 2. Use Translation Keys

Replace hardcoded strings with translation keys:

```svelte
<!-- ❌ Bad: Hardcoded text -->
<h1>Welcome to our application</h1>
<p>Please enter your email address</p>

<!-- ✅ Good: Translation keys -->
<h1>{$_('welcome.title')}</h1>
<p>{$_('auth.emailLabel')}</p>
```

### 3. Page Titles and Meta

Always use translation keys for page titles and descriptions:

```svelte
<svelte:head>
	<title>{$_('profile.pageTitle')}</title>
	<meta name="description" content={$_('profile.pageDescription')} />
</svelte:head>
```

## 🏗️ Translation Key Guidelines

### Naming Convention

Use **flat structure** with dot notation for lightweight frontend:

```
category.subcategory.element
```

### Categories

| Category          | Purpose              | Example                                          |
| ----------------- | -------------------- | ------------------------------------------------ |
| `common.*`        | Shared UI elements   | `common.loading`, `common.save`                  |
| `nav.*`           | Navigation elements  | `nav.home`, `nav.profile`                        |
| `auth.*`          | Authentication pages | `auth.login.title`, `auth.register.submit`       |
| `profile.*`       | Profile pages        | `profile.title`, `profile.settings`              |
| `error.*`         | Error messages       | `error.notFound`, `error.serverError`            |
| `validation.*`    | Form validation      | `validation.required`, `validation.emailInvalid` |
| `accessibility.*` | Screen reader text   | `accessibility.skipToMain`                       |

### Examples

```json
{
	"common": {
		"loading": "Loading...",
		"save": "Save",
		"cancel": "Cancel",
		"delete": "Delete",
		"edit": "Edit"
	},
	"auth": {
		"login": {
			"title": "Sign in to your account",
			"submit": "Sign In",
			"pageTitle": "Sign In - Login to Your Account"
		}
	},
	"validation": {
		"required": "This field is required",
		"emailInvalid": "Please enter a valid email address"
	}
}
```

## 🌐 Adding New Languages

### Step 1: Add Locale Configuration

Update `src/lib/i18n/index.ts`:

```typescript
export const SUPPORTED_LOCALES = [
	'en-US',
	'es-ES',
	'zh-CN',
	'ar-SA',
	'fr-FR' // Add new locale
] as const;
```

### Step 2: Create Translation File

Create `src/lib/i18n/locales/fr-FR.json`:

```json
{
	"common.loading": "Chargement...",
	"nav.home": "Accueil",
	"auth.login.title": "Connectez-vous à votre compte"
}
```

### Step 3: Add to Language Selector

Update `src/lib/stores/locale.ts`:

```typescript
const localeNames: Record<SupportedLocale, string> = {
	'en-US': 'English',
	'es-ES': 'Español',
	'zh-CN': '中文',
	'ar-SA': 'العربية',
	'fr-FR': 'Français' // Add new language
};
```

### Step 4: Configure RTL (if needed)

If adding an RTL language, update `src/lib/stores/locale.ts`:

```typescript
export const isRTL = derived(locale, ($locale) => {
	const rtlLocales: SupportedLocale[] = ['ar-SA', 'he-IL']; // Add RTL locales
	return rtlLocales.includes($locale);
});
```

## 🧩 Component Implementation

### Standard Component

```svelte
<script lang="ts">
	import { _ } from 'svelte-i18n';

	// Your component logic
</script>

<div>
	<h2>{$_('component.title')}</h2>
	<p>{$_('component.description')}</p>
	<button>{$_('common.save')}</button>
</div>
```

### Form Component with Validation

```svelte
<script lang="ts">
	import { _ } from 'svelte-i18n';

	let email = '';
	let error = '';

	function validate() {
		if (!email) {
			error = $_('validation.required');
		} else if (!isValidEmail(email)) {
			error = $_('validation.emailInvalid');
		}
	}
</script>

<form>
	<label for="email">{$_('auth.emailLabel')}</label>
	<input
		id="email"
		bind:value={email}
		placeholder={$_('auth.emailPlaceholder')}
		on:blur={validate}
	/>
	{#if error}
		<p class="error">{error}</p>
	{/if}
</form>
```

## 📄 Page Implementation

### Required Implementation

Every page **MUST** include:

1. **Translation Import**
2. **Page Title**
3. **Meta Description**
4. **All User-Facing Text**

### Page Template

```svelte
<script lang="ts">
	import { _ } from 'svelte-i18n';

	// Page logic here
</script>

<svelte:head>
	<title>{$_('pageName.pageTitle')}</title>
	<meta name="description" content={$_('pageName.pageDescription')} />
</svelte:head>

<main>
	<h1>{$_('pageName.title')}</h1>
	<p>{$_('pageName.description')}</p>

	<!-- All text must use translation keys -->
</main>
```

### Page-Specific Translation Keys

Each page should have these standard keys:

```json
{
	"pageName.pageTitle": "Page Title - Site Name",
	"pageName.pageDescription": "SEO description for search engines",
	"pageName.title": "Main page heading",
	"pageName.description": "Page description or subtitle"
}
```

## 🧪 Testing & Validation

### E2E Tests

Create Playwright tests for language switching:

```typescript
// e2e/i18n-[page].test.ts
import { test, expect } from '@playwright/test';

test.describe('Page Name i18n', () => {
	test('should display content in all languages', async ({ page }) => {
		await page.goto('/page-url');

		// Test English (default)
		await expect(page.getByText('English text')).toBeVisible();

		// Switch to Spanish
		await page.locator('#language-select').selectOption('es-ES');
		await expect(page.getByText('Spanish text')).toBeVisible();

		// Test page title
		await expect(page).toHaveTitle('Spanish Page Title');
	});
});
```

### Translation Key Validation

Use our validation script to check for missing translations:

```bash
# Run i18n validation test
bun test i18n-validation
```

### Manual Testing Checklist

- [ ] All text displays correctly in each language
- [ ] Page titles and meta descriptions are translated
- [ ] Form labels and validation messages work
- [ ] RTL layout renders correctly for Arabic
- [ ] Language persistence works across navigation

## 🔄 RTL Language Support

### Automatic Direction Switching

RTL languages automatically set `dir="rtl"` on the `<html>` element:

```typescript
// This happens automatically
$: document.documentElement.dir = $isRTL ? 'rtl' : 'ltr';
$: document.documentElement.lang = $locale;
```

### RTL-Aware CSS

Use Tailwind's RTL utilities for directional styling:

```html
<!-- Margins that flip for RTL -->
<div class="ml-4 rtl:mr-4 rtl:ml-0">Content</div>

<!-- Text alignment -->
<p class="text-left rtl:text-right">Text content</p>
```

### Testing RTL

Always test RTL languages (Arabic) to ensure:

- Text flows right-to-left
- UI elements are properly mirrored
- Icons and symbols display correctly
- Form layouts work in RTL

## 🐛 Troubleshooting

### Common Issues

#### Translation Key Not Found

```
Error: Translation key 'some.key' not found
```

**Solution**: Add the missing key to all translation files:

```json
// Add to all locale files (en-US.json, es-ES.json, etc.)
{
	"some.key": "Translated text"
}
```

#### Loading State Shows Before Translation

```svelte
<!-- Problem: Shows "Loading..." before i18n loads -->
<p>Loading...</p>

<!-- Solution: Use translation with fallback -->
<p>{$_('common.loading') || 'Loading...'}</p>
```

#### TypeScript Errors

```typescript
// Error: Type 'string | undefined' is not assignable to type 'string'

// Solution: Provide fallback
const title = $_('page.title') || 'Default Title';
```

### Debug Mode

Enable i18n debugging in development:

```typescript
// src/lib/i18n/index.ts
init({
	fallbackLocale: DEFAULT_LOCALE,
	initialLocale: getInitialLocale(),
	warnOnMissingMessages: true // Enable warnings
});
```

## 📝 Developer Workflow

### Before Creating a New Page

1. **Plan Translation Keys**: Identify all user-facing text
2. **Add Translation Keys**: Add to all locale files
3. **Implement Page**: Use translation keys from the start
4. **Test All Languages**: Verify content in each language
5. **Write E2E Tests**: Test language switching functionality

### Before Submitting PR

1. **Run Validation**: `bun test i18n-validation`
2. **Test Language Switching**: Manually test all languages
3. **Check RTL**: Test Arabic language display
4. **Verify Meta Tags**: Ensure page titles and descriptions are translated

## 🔗 Related Files

- `src/lib/i18n/index.ts` - Main i18n configuration
- `src/lib/stores/locale.ts` - Locale management store
- `src/lib/components/LanguageSelector.svelte` - Language switching UI
- `e2e/language-switching.test.ts` - E2E tests for i18n functionality
- `documentation/I18N_PROCESS.md` - Complete implementation process

## 📞 Support

For questions about i18n implementation:

1. Check this README for common patterns
2. Review existing translated pages for examples
3. Run the i18n validation tests
4. Check the troubleshooting section

Remember: **Every user-facing string must be translatable!**
