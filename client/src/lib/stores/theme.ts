/**
 * Theme Management Store - Advanced Theming System
 *
 * Implements comprehensive theme management with:
 * - System preference detection
 * - Manual theme override (light/dark/system)
 * - Theme persistence with localStorage
 * - FOIT (Flash of Incorrect Theme) prevention
 * - Reactive theme updates across the application
 */

import { writable, derived, type Readable } from 'svelte/store';
import { browser } from '$app/environment';

// Theme types
export type Theme = 'light' | 'dark' | 'system';
export type ResolvedTheme = 'light' | 'dark';

// Theme store - user's preference (light/dark/system)
export const theme = writable<Theme>('system');

// System theme detection
export const systemTheme = writable<ResolvedTheme>('light');

// Resolved theme - the actual theme being used
export const resolvedTheme: Readable<ResolvedTheme> = derived(
	[theme, systemTheme],
	([$theme, $systemTheme]) => {
		if ($theme === 'system') {
			return $systemTheme;
		}
		return $theme as ResolvedTheme;
	}
);

// Theme initialization and management
class ThemeManager {
	private mediaQuery: MediaQueryList | null = null;
	private initialized = false;

	/**
	 * Initialize theme management
	 * Should be called once when the app starts
	 */
	init() {
		if (!browser || this.initialized) return;

		this.initialized = true;

		// Set up system theme detection
		this.setupSystemThemeDetection();

		// Load saved theme from localStorage
		this.loadSavedTheme();

		// Subscribe to theme changes to update DOM and save to localStorage
		this.setupThemeSubscription();
	}

	/**
	 * Set up system theme detection using prefers-color-scheme
	 */
	private setupSystemThemeDetection() {
		if (!window.matchMedia) return;

		this.mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

		// Set initial system theme
		systemTheme.set(this.mediaQuery.matches ? 'dark' : 'light');

		// Listen for system theme changes
		const handler = (e: MediaQueryListEvent) => {
			systemTheme.set(e.matches ? 'dark' : 'light');
		};

		// Modern event listener
		if (this.mediaQuery.addEventListener) {
			this.mediaQuery.addEventListener('change', handler);
		} else {
			// Fallback for older browsers
			this.mediaQuery.addListener(handler);
		}
	}

	/**
	 * Load saved theme preference from localStorage
	 */
	private loadSavedTheme() {
		try {
			const saved = localStorage.getItem('theme') as Theme;
			if (saved && ['light', 'dark', 'system'].includes(saved)) {
				theme.set(saved);
			}
		} catch (error) {
			console.warn('Failed to load theme from localStorage:', error);
		}
	}

	/**
	 * Set up theme subscription to update DOM and localStorage
	 */
	private setupThemeSubscription() {
		// Subscribe to resolved theme changes to update DOM
		resolvedTheme.subscribe((newTheme) => {
			this.updateDOM(newTheme);
		});

		// Subscribe to theme preference changes to save to localStorage
		theme.subscribe((newTheme) => {
			this.saveTheme(newTheme);
		});
	}

	/**
	 * Update DOM classes and attributes for theme
	 */
	private updateDOM(resolvedTheme: ResolvedTheme) {
		if (!browser) return;

		const root = document.documentElement;

		// Update class for CSS targeting
		root.classList.remove('light', 'dark');
		root.classList.add(resolvedTheme);

		// Update color-scheme for browser UI
		root.style.colorScheme = resolvedTheme;

		// Update meta theme-color for mobile browsers
		this.updateMetaThemeColor(resolvedTheme);
	}

	/**
	 * Update meta theme-color for mobile browser UI
	 */
	private updateMetaThemeColor(resolvedTheme: ResolvedTheme) {
		let metaThemeColor = document.querySelector('meta[name="theme-color"]');

		if (!metaThemeColor) {
			metaThemeColor = document.createElement('meta');
			metaThemeColor.setAttribute('name', 'theme-color');
			document.head.appendChild(metaThemeColor);
		}

		// Use our design token colors
		const color = resolvedTheme === 'dark' ? '#000000' : '#ffffff';
		metaThemeColor.setAttribute('content', color);
	}

	/**
	 * Save theme preference to localStorage
	 */
	private saveTheme(theme: Theme) {
		if (!browser) return;

		try {
			localStorage.setItem('theme', theme);
		} catch (error) {
			console.warn('Failed to save theme to localStorage:', error);
		}
	}

	/**
	 * Toggle between light and dark themes
	 * If currently on system, switches to the opposite of current system theme
	 */
	toggleTheme() {
		theme.update((current) => {
			// Get current resolved theme
			let currentResolved: ResolvedTheme;

			if (current === 'system') {
				// Check actual system preference
				currentResolved = this.mediaQuery?.matches ? 'dark' : 'light';
			} else {
				currentResolved = current as ResolvedTheme;
			}

			// Toggle to opposite
			return currentResolved === 'light' ? 'dark' : 'light';
		});
	}

	/**
	 * Set theme to a specific value
	 */
	setTheme(newTheme: Theme) {
		theme.set(newTheme);
	}

	/**
	 * Get current theme info for debugging
	 */
	getThemeInfo() {
		if (!browser) return null;

		return {
			userPreference: theme,
			systemPreference: this.mediaQuery?.matches ? 'dark' : 'light',
			resolved: document.documentElement.classList.contains('dark') ? 'dark' : 'light',
			supportsPreference: !!window.matchMedia
		};
	}

	/**
	 * Cleanup - remove event listeners
	 */
	destroy() {
		if (this.mediaQuery && this.mediaQuery.removeEventListener) {
			this.mediaQuery.removeEventListener('change', () => {});
		}
		this.initialized = false;
	}
}

// Create singleton instance
export const themeManager = new ThemeManager();

// Convenience functions for theme management
export const toggleTheme = () => themeManager.toggleTheme();
export const setTheme = (newTheme: Theme) => themeManager.setTheme(newTheme);
export const initTheme = () => themeManager.init();

// Theme utilities
export const themeUtils = {
	/**
	 * Check if current theme is dark
	 */
	isDark: derived(resolvedTheme, ($resolvedTheme) => $resolvedTheme === 'dark'),

	/**
	 * Check if current theme is light
	 */
	isLight: derived(resolvedTheme, ($resolvedTheme) => $resolvedTheme === 'light'),

	/**
	 * Check if user preference is set to system
	 */
	isSystemMode: derived(theme, ($theme) => $theme === 'system'),

	/**
	 * Get theme icon name for UI
	 */
	themeIcon: derived([theme], ([$theme]) => {
		if ($theme === 'system') {
			return 'monitor'; // or 'computer'
		}
		return $theme === 'dark' ? 'moon' : 'sun';
	}),

	/**
	 * Get next theme in cycle (light -> dark -> system -> light)
	 */
	nextTheme: derived(theme, ($theme) => {
		switch ($theme) {
			case 'light':
				return 'dark';
			case 'dark':
				return 'system';
			case 'system':
				return 'light';
			default:
				return 'light';
		}
	})
};
