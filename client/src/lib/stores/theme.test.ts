import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

// Mock $app/environment
vi.mock('$app/environment', () => ({
	browser: true
}));

// Get localStorage mock from setup
const localStorageMock = window.localStorage as unknown as {
	getItem: ReturnType<typeof vi.fn>;
	setItem: ReturnType<typeof vi.fn>;
	removeItem: ReturnType<typeof vi.fn>;
};

// Import after mocks
const { theme, systemTheme, resolvedTheme, themeManager, toggleTheme, setTheme, themeUtils } =
	await import('./theme');

describe('theme store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		// Reset stores to defaults
		theme.set('system');
		systemTheme.set('light');
		localStorageMock.getItem.mockReturnValue(null);
	});

	describe('theme store', () => {
		it('should have initial value of system', () => {
			theme.set('system');
			expect(get(theme)).toBe('system');
		});

		it('should accept light, dark, and system values', () => {
			theme.set('light');
			expect(get(theme)).toBe('light');

			theme.set('dark');
			expect(get(theme)).toBe('dark');

			theme.set('system');
			expect(get(theme)).toBe('system');
		});
	});

	describe('systemTheme store', () => {
		it('should store system preference', () => {
			systemTheme.set('dark');
			expect(get(systemTheme)).toBe('dark');

			systemTheme.set('light');
			expect(get(systemTheme)).toBe('light');
		});
	});

	describe('resolvedTheme', () => {
		it('should return light when theme is light', () => {
			theme.set('light');
			expect(get(resolvedTheme)).toBe('light');
		});

		it('should return dark when theme is dark', () => {
			theme.set('dark');
			expect(get(resolvedTheme)).toBe('dark');
		});

		it('should follow system preference when theme is system', () => {
			theme.set('system');

			systemTheme.set('light');
			expect(get(resolvedTheme)).toBe('light');

			systemTheme.set('dark');
			expect(get(resolvedTheme)).toBe('dark');
		});
	});

	describe('themeManager', () => {
		describe('toggleTheme', () => {
			it('should toggle from light to dark', () => {
				theme.set('light');
				themeManager.toggleTheme();
				expect(get(theme)).toBe('dark');
			});

			it('should toggle from dark to light', () => {
				theme.set('dark');
				themeManager.toggleTheme();
				expect(get(theme)).toBe('light');
			});

			it('should toggle from system to opposite of current system theme', () => {
				theme.set('system');
				systemTheme.set('light');

				themeManager.toggleTheme();
				expect(get(theme)).toBe('dark');
			});
		});

		describe('setTheme', () => {
			it('should set theme to specified value', () => {
				themeManager.setTheme('dark');
				expect(get(theme)).toBe('dark');

				themeManager.setTheme('light');
				expect(get(theme)).toBe('light');

				themeManager.setTheme('system');
				expect(get(theme)).toBe('system');
			});
		});
	});

	describe('toggleTheme function', () => {
		it('should toggle theme like themeManager.toggleTheme', () => {
			theme.set('light');
			toggleTheme();
			expect(get(theme)).toBe('dark');
		});
	});

	describe('setTheme function', () => {
		it('should set theme like themeManager.setTheme', () => {
			setTheme('dark');
			expect(get(theme)).toBe('dark');
		});
	});

	describe('themeUtils', () => {
		describe('isDark', () => {
			it('should be true when resolved theme is dark', () => {
				theme.set('dark');
				expect(get(themeUtils.isDark)).toBe(true);
			});

			it('should be false when resolved theme is light', () => {
				theme.set('light');
				expect(get(themeUtils.isDark)).toBe(false);
			});
		});

		describe('isLight', () => {
			it('should be true when resolved theme is light', () => {
				theme.set('light');
				expect(get(themeUtils.isLight)).toBe(true);
			});

			it('should be false when resolved theme is dark', () => {
				theme.set('dark');
				expect(get(themeUtils.isLight)).toBe(false);
			});
		});

		describe('isSystemMode', () => {
			it('should be true when theme is system', () => {
				theme.set('system');
				expect(get(themeUtils.isSystemMode)).toBe(true);
			});

			it('should be false when theme is not system', () => {
				theme.set('light');
				expect(get(themeUtils.isSystemMode)).toBe(false);

				theme.set('dark');
				expect(get(themeUtils.isSystemMode)).toBe(false);
			});
		});

		describe('themeIcon', () => {
			it('should return sun for light theme', () => {
				theme.set('light');
				expect(get(themeUtils.themeIcon)).toBe('sun');
			});

			it('should return moon for dark theme', () => {
				theme.set('dark');
				expect(get(themeUtils.themeIcon)).toBe('moon');
			});

			it('should return monitor for system theme', () => {
				theme.set('system');
				expect(get(themeUtils.themeIcon)).toBe('monitor');
			});
		});

		describe('nextTheme', () => {
			it('should return dark when current is light', () => {
				theme.set('light');
				expect(get(themeUtils.nextTheme)).toBe('dark');
			});

			it('should return system when current is dark', () => {
				theme.set('dark');
				expect(get(themeUtils.nextTheme)).toBe('system');
			});

			it('should return light when current is system', () => {
				theme.set('system');
				expect(get(themeUtils.nextTheme)).toBe('light');
			});
		});
	});

	describe('persistence', () => {
		it('should save theme to localStorage when changed', () => {
			// Initialize the theme manager to set up subscriptions
			themeManager.init();

			theme.set('dark');

			// The subscription should have been triggered
			expect(localStorageMock.setItem).toHaveBeenCalledWith('theme', 'dark');
		});
	});
});
