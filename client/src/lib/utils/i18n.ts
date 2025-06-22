import { _ } from 'svelte-i18n';
import { get } from 'svelte/store';
import type { TranslationKey } from '$lib/i18n/types';

/**
 * Type-safe translation function
 * Provides compile-time checking for translation keys
 */
export function t(key: TranslationKey, params?: Record<string, string | number>): string {
	const translator = get(_);
	return translator(key, { values: params }) || key;
}

/**
 * Format number based on current locale
 */
export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
	// This will be enhanced when we have access to current locale
	return new Intl.NumberFormat('en-US', options).format(value);
}

/**
 * Format date based on current locale
 */
export function formatDate(date: Date | string, options?: Intl.DateTimeFormatOptions): string {
	const dateObj = typeof date === 'string' ? new Date(date) : date;
	// This will be enhanced when we have access to current locale
	return new Intl.DateTimeFormat('en-US', options).format(dateObj);
}

/**
 * Format relative time (e.g., "2 hours ago")
 */
export function formatRelativeTime(date: Date | string): string {
	const dateObj = typeof date === 'string' ? new Date(date) : date;
	const now = new Date();
	const diffInSeconds = Math.floor((now.getTime() - dateObj.getTime()) / 1000);

	const intervals = [
		{ label: 'year', seconds: 31536000 },
		{ label: 'month', seconds: 2592000 },
		{ label: 'week', seconds: 604800 },
		{ label: 'day', seconds: 86400 },
		{ label: 'hour', seconds: 3600 },
		{ label: 'minute', seconds: 60 },
		{ label: 'second', seconds: 1 }
	];

	for (const interval of intervals) {
		const count = Math.floor(diffInSeconds / interval.seconds);
		if (count >= 1) {
			// This will be enhanced with proper i18n when relative time translations are added
			return `${count} ${interval.label}${count > 1 ? 's' : ''} ago`;
		}
	}

	return 'just now';
}

/**
 * Get direction for text based on language code
 */
export function getTextDirection(langCode: string): 'ltr' | 'rtl' {
	const rtlLanguages = ['ar', 'he', 'fa', 'ur'];
	const lang = langCode.split('-')[0].toLowerCase();
	return rtlLanguages.includes(lang) ? 'rtl' : 'ltr';
}

/**
 * Check if a language uses RTL direction
 */
export function isRTL(langCode: string): boolean {
	return getTextDirection(langCode) === 'rtl';
}

/**
 * Validate that all required translation keys exist in a translation object
 */
export function validateTranslations(
	translations: Record<string, unknown>,
	requiredKeys: string[]
): string[] {
	const missingKeys: string[] = [];

	for (const key of requiredKeys) {
		if (!(key in translations)) {
			missingKeys.push(key);
		}
	}

	return missingKeys;
}

/**
 * Helper to interpolate values into translation strings
 * Used when svelte-i18n's built-in interpolation isn't available
 */
export function interpolate(template: string, values: Record<string, string | number>): string {
	return template.replace(/\{(\w+)\}/g, (match, key) => {
		return key in values ? String(values[key]) : match;
	});
}
