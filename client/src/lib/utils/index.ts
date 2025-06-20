import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Utility function to merge Tailwind CSS classes with clsx
 * This ensures proper class precedence and removes conflicts
 */
export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// Re-export accessibility utilities
export { a11y } from './accessibility.js';

/**
 * Generate a unique ID for components
 */
export function generateId(): string {
	return Math.random().toString(36).substring(2, 9);
}

/**
 * Debounce function for performance optimization
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
	func: T,
	wait: number
): (...args: Parameters<T>) => void {
	let timeout: ReturnType<typeof setTimeout>;
	return (...args: Parameters<T>) => {
		clearTimeout(timeout);
		timeout = setTimeout(() => func(...args), wait);
	};
}

/**
 * Check if an element is currently in the viewport
 */
export function isInViewport(element: HTMLElement): boolean {
	const rect = element.getBoundingClientRect();
	return (
		rect.top >= 0 &&
		rect.left >= 0 &&
		rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
		rect.right <= (window.innerWidth || document.documentElement.clientWidth)
	);
}

/**
 * Format a date string for display
 */
export function formatDate(date: string | Date): string {
	const d = new Date(date);
	return new Intl.DateTimeFormat('en-US', {
		year: 'numeric',
		month: 'long',
		day: 'numeric'
	}).format(d);
}

/**
 * Format a relative time string (e.g., "2 hours ago")
 */
export function formatRelativeTime(date: string | Date): string {
	const now = new Date();
	const d = new Date(date);
	const diffInSeconds = Math.floor((now.getTime() - d.getTime()) / 1000);

	const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });

	if (diffInSeconds < 60) {
		return rtf.format(-diffInSeconds, 'second');
	} else if (diffInSeconds < 3600) {
		return rtf.format(-Math.floor(diffInSeconds / 60), 'minute');
	} else if (diffInSeconds < 86400) {
		return rtf.format(-Math.floor(diffInSeconds / 3600), 'hour');
	} else if (diffInSeconds < 2592000) {
		return rtf.format(-Math.floor(diffInSeconds / 86400), 'day');
	} else if (diffInSeconds < 31536000) {
		return rtf.format(-Math.floor(diffInSeconds / 2592000), 'month');
	} else {
		return rtf.format(-Math.floor(diffInSeconds / 31536000), 'year');
	}
}
