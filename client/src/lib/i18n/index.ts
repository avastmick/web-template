import { register, init, getLocaleFromNavigator, locale as svelteI18nLocale } from 'svelte-i18n';
import { derived, get } from 'svelte/store';

// Define supported locales
export const SUPPORTED_LOCALES = ['en-US', 'es-ES', 'zh-CN', 'ar-SA'] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

export const DEFAULT_LOCALE: SupportedLocale = 'en-US';

// Language metadata
export const LOCALE_NAMES: Record<SupportedLocale, string> = {
	'en-US': 'English',
	'es-ES': 'Español',
	'zh-CN': '中文',
	'ar-SA': 'العربية'
};

// RTL languages
const RTL_LOCALES: SupportedLocale[] = ['ar-SA'];

// Register translations with lazy loading
SUPPORTED_LOCALES.forEach((lng) => {
	register(lng, () => import(`./locales/${lng}.json`));
});

// Create RTL direction store
export const dir = derived(svelteI18nLocale, ($locale) =>
	RTL_LOCALES.includes($locale as SupportedLocale) ? 'rtl' : 'ltr'
);

// Locale persistence
if (typeof window !== 'undefined') {
	svelteI18nLocale.subscribe((value) => {
		if (value) {
			localStorage.setItem('locale', value);
		}
	});
}

// Initialize i18n with locale detection
export function initializeI18n(): void {
	// Check for saved locale in localStorage
	const savedLocale = typeof window !== 'undefined' ? localStorage.getItem('locale') : null;

	// Get browser locale
	const browserLocale = getLocaleFromNavigator();

	// Determine initial locale
	let initialLocale: SupportedLocale = DEFAULT_LOCALE;

	if (savedLocale && SUPPORTED_LOCALES.includes(savedLocale as SupportedLocale)) {
		initialLocale = savedLocale as SupportedLocale;
	} else if (browserLocale) {
		// Check for exact match
		if (SUPPORTED_LOCALES.includes(browserLocale as SupportedLocale)) {
			initialLocale = browserLocale as SupportedLocale;
		} else {
			// Check for language code match (e.g., 'en' matches 'en-US')
			const browserLang = browserLocale.split('-')[0];
			const match = SUPPORTED_LOCALES.find((loc) => loc.startsWith(browserLang));
			if (match) {
				initialLocale = match;
			}
		}
	}

	init({
		fallbackLocale: DEFAULT_LOCALE,
		initialLocale
	});
}

// Helper to check if a locale is supported
export function isSupportedLocale(locale: string): locale is SupportedLocale {
	return SUPPORTED_LOCALES.includes(locale as SupportedLocale);
}

// Helper to get current locale
export function getCurrentLocale(): SupportedLocale {
	const currentLocale = get(svelteI18nLocale);
	return currentLocale && isSupportedLocale(currentLocale) ? currentLocale : DEFAULT_LOCALE;
}
