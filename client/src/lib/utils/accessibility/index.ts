/**
 * Accessibility Utilities - WCAG 2.1 AA Compliance
 *
 * Modular accessibility utilities for implementing WCAG 2.1 AA standards
 */

export { WCAG_STANDARDS } from './constants.js';
export { ColorContrast } from './color-contrast.js';
export { FocusManagement } from './focus-management.js';
export { TouchTargets } from './touch-targets.js';
export { A11yTesting } from './testing.js';

// Screen Reader, Motion, and Keyboard utilities (kept in main file for brevity)
import { browser } from '$app/environment';
import { WCAG_STANDARDS } from './constants.js';
import { ColorContrast } from './color-contrast.js';
import { FocusManagement } from './focus-management.js';
import { TouchTargets } from './touch-targets.js';
import { A11yTesting } from './testing.js';

/**
 * Screen Reader Utilities
 */
export class ScreenReader {
	static announce(text: string, priority: 'polite' | 'assertive' = 'polite'): void {
		if (!browser) return;

		const announcer = document.createElement('div');
		announcer.setAttribute('aria-live', priority);
		announcer.setAttribute('aria-atomic', 'true');
		announcer.setAttribute('class', 'sr-only');
		announcer.textContent = text;

		document.body.appendChild(announcer);
		setTimeout(() => document.body.removeChild(announcer), 1000);
	}

	static updateAccessibleName(element: HTMLElement, name: string): void {
		element.setAttribute('aria-label', name);
	}
}

/**
 * Motion and Animation Utilities
 */
export class MotionUtils {
	static prefersReducedMotion(): boolean {
		if (!browser) return false;
		return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
	}

	static getAnimationDuration(defaultDuration: number): number {
		return this.prefersReducedMotion()
			? Math.min(defaultDuration, WCAG_STANDARDS.REDUCED_MOTION_DURATION)
			: defaultDuration;
	}
}

/**
 * Keyboard Navigation Utilities
 */
export class KeyboardNavigation {
	static handleArrowNavigation(
		event: KeyboardEvent,
		elements: HTMLElement[],
		currentIndex: number,
		options: { horizontal?: boolean; vertical?: boolean; wrap?: boolean } = {}
	): number {
		const { horizontal = true, vertical = true, wrap = true } = options;
		let newIndex = currentIndex;

		switch (event.key) {
			case 'ArrowUp':
				if (vertical) {
					event.preventDefault();
					newIndex =
						currentIndex > 0 ? currentIndex - 1 : wrap ? elements.length - 1 : currentIndex;
				}
				break;
			case 'ArrowDown':
				if (vertical) {
					event.preventDefault();
					newIndex =
						currentIndex < elements.length - 1 ? currentIndex + 1 : wrap ? 0 : currentIndex;
				}
				break;
			case 'ArrowLeft':
				if (horizontal) {
					event.preventDefault();
					newIndex =
						currentIndex > 0 ? currentIndex - 1 : wrap ? elements.length - 1 : currentIndex;
				}
				break;
			case 'ArrowRight':
				if (horizontal) {
					event.preventDefault();
					newIndex =
						currentIndex < elements.length - 1 ? currentIndex + 1 : wrap ? 0 : currentIndex;
				}
				break;
			case 'Home':
				event.preventDefault();
				newIndex = 0;
				break;
			case 'End':
				event.preventDefault();
				newIndex = elements.length - 1;
				break;
		}

		if (newIndex !== currentIndex && elements[newIndex]) {
			elements[newIndex].focus();
		}

		return newIndex;
	}

	static addRoleNavigation(
		container: HTMLElement,
		role: 'menu' | 'tablist' | 'listbox'
	): () => void {
		const elements = FocusManagement.getFocusableElements(container);
		let currentIndex = 0;

		const handleKeyDown = (event: KeyboardEvent) => {
			const target = event.target as HTMLElement;
			const targetIndex = elements.indexOf(target);

			if (targetIndex !== -1) {
				currentIndex = targetIndex;
			}

			switch (role) {
				case 'menu':
					currentIndex = this.handleArrowNavigation(event, elements, currentIndex, {
						horizontal: false,
						vertical: true
					});
					break;
				case 'tablist':
					currentIndex = this.handleArrowNavigation(event, elements, currentIndex, {
						horizontal: true,
						vertical: false
					});
					break;
				case 'listbox':
					currentIndex = this.handleArrowNavigation(event, elements, currentIndex);
					break;
			}
		};

		container.addEventListener('keydown', handleKeyDown);
		container.setAttribute('role', role);

		return () => {
			container.removeEventListener('keydown', handleKeyDown);
		};
	}
}

/**
 * Global accessibility utilities export
 */
export const a11y = {
	ColorContrast,
	FocusManagement,
	ScreenReader,
	MotionUtils,
	TouchTargets,
	KeyboardNavigation,
	A11yTesting,
	WCAG_STANDARDS
};
