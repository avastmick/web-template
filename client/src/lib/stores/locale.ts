import { writable, derived, get } from 'svelte/store';
import { locale as svelteI18nLocale } from 'svelte-i18n';
import type { SupportedLocale } from '$lib/i18n';
import { SUPPORTED_LOCALES, DEFAULT_LOCALE, LOCALE_NAMES } from '$lib/i18n';

// Create a typed locale store that syncs with svelte-i18n
function createLocaleStore() {
	const { subscribe, set } = writable<SupportedLocale>(DEFAULT_LOCALE);

	// Sync with svelte-i18n locale store
	svelteI18nLocale.subscribe((value) => {
		if (value && SUPPORTED_LOCALES.includes(value as SupportedLocale)) {
			set(value as SupportedLocale);
		}
	});

	return {
		subscribe,
		set: (value: SupportedLocale) => {
			if (SUPPORTED_LOCALES.includes(value)) {
				set(value);
				svelteI18nLocale.set(value);
				// Persist to localStorage
				if (typeof window !== 'undefined') {
					localStorage.setItem('locale', value);
				}
			}
		},
		reset: () => {
			set(DEFAULT_LOCALE);
			svelteI18nLocale.set(DEFAULT_LOCALE);
			if (typeof window !== 'undefined') {
				localStorage.removeItem('locale');
			}
		}
	};
}

export const locale = createLocaleStore();

// Derived stores for convenience
export const localeName = derived(locale, ($locale) => LOCALE_NAMES[$locale]);

export const isRTL = derived(locale, ($locale) => {
	const rtlLocales: SupportedLocale[] = ['ar-SA'];
	return rtlLocales.includes($locale);
});

export const direction = derived(isRTL, ($isRTL) => ($isRTL ? 'rtl' : 'ltr'));

// Helper functions
export function setLocale(newLocale: SupportedLocale): void {
	locale.set(newLocale);
}

export function getCurrentLocale(): SupportedLocale {
	return get(locale);
}

export function getLocaleName(localeCode?: SupportedLocale): string {
	const targetLocale = localeCode || getCurrentLocale();
	return LOCALE_NAMES[targetLocale] || targetLocale;
}

export function getAllLocales(): Array<{ code: SupportedLocale; name: string }> {
	return SUPPORTED_LOCALES.map((code) => ({
		code,
		name: LOCALE_NAMES[code]
	}));
}

// Browser language detection
export function detectBrowserLocale(): SupportedLocale {
	if (typeof window === 'undefined') {
		return DEFAULT_LOCALE;
	}

	// Get browser languages in order of preference
	const browserLanguages = navigator.languages || [navigator.language];

	for (const browserLang of browserLanguages) {
		// Check for exact match
		if (SUPPORTED_LOCALES.includes(browserLang as SupportedLocale)) {
			return browserLang as SupportedLocale;
		}

		// Check for language code match (e.g., 'en' matches 'en-US')
		const langCode = browserLang.split('-')[0];
		const match = SUPPORTED_LOCALES.find((supportedLocale) => supportedLocale.startsWith(langCode));

		if (match) {
			return match;
		}
	}

	return DEFAULT_LOCALE;
}

// Locale persistence
export function getStoredLocale(): SupportedLocale | null {
	if (typeof window === 'undefined') {
		return null;
	}

	const stored = localStorage.getItem('locale');
	if (stored && SUPPORTED_LOCALES.includes(stored as SupportedLocale)) {
		return stored as SupportedLocale;
	}

	return null;
}

// Initialize locale from storage or browser detection
export function initializeLocale(): SupportedLocale {
	// Priority: stored > browser > default
	const storedLocale = getStoredLocale();
	if (storedLocale) {
		return storedLocale;
	}

	const browserLocale = detectBrowserLocale();
	return browserLocale;
}

// Reactive locale effects
if (typeof window !== 'undefined') {
	// Apply direction to document
	direction.subscribe((dir) => {
		document.documentElement.dir = dir;
	});

	// Apply language to document
	locale.subscribe((loc) => {
		document.documentElement.lang = loc;
	});
}
