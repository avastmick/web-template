import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get, writable } from 'svelte/store';

// Mock svelte-i18n
const mockSvelteI18nLocale = writable<string | null>('en-US');

vi.mock('svelte-i18n', () => ({
	locale: mockSvelteI18nLocale
}));

// Mock $lib/i18n
vi.mock('$lib/i18n', () => ({
	SUPPORTED_LOCALES: ['en-US', 'es-ES', 'zh-CN', 'ar-SA'] as const,
	DEFAULT_LOCALE: 'en-US',
	LOCALE_NAMES: {
		'en-US': 'English',
		'es-ES': 'Español',
		'zh-CN': '中文',
		'ar-SA': 'العربية'
	}
}));

// Get localStorage mock from setup
const localStorageMock = window.localStorage as unknown as {
	getItem: ReturnType<typeof vi.fn>;
	setItem: ReturnType<typeof vi.fn>;
	removeItem: ReturnType<typeof vi.fn>;
};

// Import after mocks
const {
	locale,
	localeName,
	isRTL,
	direction,
	setLocale,
	getCurrentLocale,
	getLocaleName,
	getAllLocales,
	detectBrowserLocale,
	getStoredLocale,
	initializeLocale
} = await import('./locale');

describe('locale store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorageMock.getItem.mockReturnValue(null);
		mockSvelteI18nLocale.set('en-US');
	});

	describe('locale store', () => {
		it('should have default locale', () => {
			expect(get(locale)).toBe('en-US');
		});

		it('should set locale', () => {
			locale.set('es-ES');
			expect(get(locale)).toBe('es-ES');
		});

		it('should only accept supported locales', () => {
			locale.set('es-ES');
			expect(get(locale)).toBe('es-ES');

			locale.set('zh-CN');
			expect(get(locale)).toBe('zh-CN');

			locale.set('ar-SA');
			expect(get(locale)).toBe('ar-SA');
		});

		it('should persist locale to localStorage', () => {
			locale.set('es-ES');
			expect(localStorageMock.setItem).toHaveBeenCalledWith('locale', 'es-ES');
		});

		it('should sync with svelte-i18n locale', () => {
			locale.set('zh-CN');
			expect(get(mockSvelteI18nLocale)).toBe('zh-CN');
		});
	});

	describe('localeName derived store', () => {
		it('should return locale display name', () => {
			locale.set('en-US');
			expect(get(localeName)).toBe('English');

			locale.set('es-ES');
			expect(get(localeName)).toBe('Español');

			locale.set('zh-CN');
			expect(get(localeName)).toBe('中文');

			locale.set('ar-SA');
			expect(get(localeName)).toBe('العربية');
		});
	});

	describe('isRTL derived store', () => {
		it('should be true for RTL locales', () => {
			locale.set('ar-SA');
			expect(get(isRTL)).toBe(true);
		});

		it('should be false for LTR locales', () => {
			locale.set('en-US');
			expect(get(isRTL)).toBe(false);

			locale.set('es-ES');
			expect(get(isRTL)).toBe(false);

			locale.set('zh-CN');
			expect(get(isRTL)).toBe(false);
		});
	});

	describe('direction derived store', () => {
		it('should return rtl for RTL locales', () => {
			locale.set('ar-SA');
			expect(get(direction)).toBe('rtl');
		});

		it('should return ltr for LTR locales', () => {
			locale.set('en-US');
			expect(get(direction)).toBe('ltr');
		});
	});

	describe('setLocale function', () => {
		it('should set locale', () => {
			setLocale('es-ES');
			expect(get(locale)).toBe('es-ES');
		});
	});

	describe('getCurrentLocale function', () => {
		it('should return current locale', () => {
			locale.set('zh-CN');
			expect(getCurrentLocale()).toBe('zh-CN');
		});
	});

	describe('getLocaleName function', () => {
		it('should return name for specified locale', () => {
			expect(getLocaleName('es-ES')).toBe('Español');
			expect(getLocaleName('zh-CN')).toBe('中文');
		});

		it('should return name for current locale if not specified', () => {
			locale.set('ar-SA');
			expect(getLocaleName()).toBe('العربية');
		});
	});

	describe('getAllLocales function', () => {
		it('should return all supported locales with names', () => {
			const locales = getAllLocales();

			expect(locales).toHaveLength(4);
			expect(locales).toContainEqual({ code: 'en-US', name: 'English' });
			expect(locales).toContainEqual({ code: 'es-ES', name: 'Español' });
			expect(locales).toContainEqual({ code: 'zh-CN', name: '中文' });
			expect(locales).toContainEqual({ code: 'ar-SA', name: 'العربية' });
		});
	});

	describe('detectBrowserLocale function', () => {
		it('should return default locale when navigator.languages is empty', () => {
			Object.defineProperty(navigator, 'languages', {
				value: [],
				writable: true
			});
			Object.defineProperty(navigator, 'language', {
				value: '',
				writable: true
			});

			expect(detectBrowserLocale()).toBe('en-US');
		});

		it('should return exact match if available', () => {
			Object.defineProperty(navigator, 'languages', {
				value: ['es-ES', 'en-US'],
				writable: true
			});

			expect(detectBrowserLocale()).toBe('es-ES');
		});

		it('should match by language code prefix', () => {
			Object.defineProperty(navigator, 'languages', {
				value: ['es-MX', 'en'],
				writable: true
			});

			// Should match es-ES because es-MX starts with 'es'
			expect(detectBrowserLocale()).toBe('es-ES');
		});

		it('should return default if no match found', () => {
			Object.defineProperty(navigator, 'languages', {
				value: ['fr-FR', 'de-DE'],
				writable: true
			});

			expect(detectBrowserLocale()).toBe('en-US');
		});
	});

	describe('getStoredLocale function', () => {
		it('should return stored locale if valid', () => {
			localStorageMock.getItem.mockReturnValue('es-ES');
			expect(getStoredLocale()).toBe('es-ES');
		});

		it('should return null if stored locale is invalid', () => {
			localStorageMock.getItem.mockReturnValue('invalid-locale');
			expect(getStoredLocale()).toBeNull();
		});

		it('should return null if nothing stored', () => {
			localStorageMock.getItem.mockReturnValue(null);
			expect(getStoredLocale()).toBeNull();
		});
	});

	describe('initializeLocale function', () => {
		it('should prioritize stored locale', () => {
			localStorageMock.getItem.mockReturnValue('zh-CN');
			Object.defineProperty(navigator, 'languages', {
				value: ['es-ES'],
				writable: true
			});

			expect(initializeLocale()).toBe('zh-CN');
		});

		it('should fall back to browser locale if no stored locale', () => {
			localStorageMock.getItem.mockReturnValue(null);
			Object.defineProperty(navigator, 'languages', {
				value: ['es-ES'],
				writable: true
			});

			expect(initializeLocale()).toBe('es-ES');
		});

		it('should fall back to default if no stored or browser locale', () => {
			localStorageMock.getItem.mockReturnValue(null);
			Object.defineProperty(navigator, 'languages', {
				value: ['fr-FR'],
				writable: true
			});

			expect(initializeLocale()).toBe('en-US');
		});
	});

	describe('locale reset', () => {
		it('should reset to default locale', () => {
			locale.set('es-ES');
			locale.reset();

			expect(get(locale)).toBe('en-US');
		});

		it('should remove locale from localStorage', () => {
			locale.set('es-ES');
			locale.reset();

			expect(localStorageMock.removeItem).toHaveBeenCalledWith('locale');
		});
	});
});
