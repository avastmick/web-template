/**
 * Accessibility Utilities - WCAG 2.1 AA Compliance
 *
 * Provides utilities for implementing WCAG 2.1 AA accessibility standards:
 * - Focus management and keyboard navigation
 * - Screen reader support
 * - Color contrast validation
 * - Accessible motion and animation
 * - Touch target sizing
 */

import { browser } from '$app/environment';

// WCAG 2.1 AA standards
export const WCAG_STANDARDS = {
	// Minimum contrast ratios
	CONTRAST_RATIO_NORMAL: 4.5, // For normal text (18px and below)
	CONTRAST_RATIO_LARGE: 3.0, // For large text (18px+ or 14px+ bold)

	// Touch target minimum size (44x44px)
	TOUCH_TARGET_MIN: 44,

	// Animation and motion
	REDUCED_MOTION_DURATION: 150, // Maximum duration when prefers-reduced-motion
	DEFAULT_MOTION_DURATION: 300
};

/**
 * Color Contrast Utilities
 */
export class ColorContrast {
	/**
	 * Convert hex color to RGB
	 */
	static hexToRgb(hex: string): { r: number; g: number; b: number } | null {
		const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
		return result
			? {
					r: parseInt(result[1], 16),
					g: parseInt(result[2], 16),
					b: parseInt(result[3], 16)
				}
			: null;
	}

	/**
	 * Calculate relative luminance of a color
	 * https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html
	 */
	static getRelativeLuminance(r: number, g: number, b: number): number {
		const getRGB = (value: number) => {
			const sRGB = value / 255;
			return sRGB <= 0.03928 ? sRGB / 12.92 : Math.pow((sRGB + 0.055) / 1.055, 2.4);
		};

		return 0.2126 * getRGB(r) + 0.7152 * getRGB(g) + 0.0722 * getRGB(b);
	}

	/**
	 * Calculate contrast ratio between two colors
	 */
	static getContrastRatio(color1: string, color2: string): number | null {
		const rgb1 = this.hexToRgb(color1);
		const rgb2 = this.hexToRgb(color2);

		if (!rgb1 || !rgb2) return null;

		const lum1 = this.getRelativeLuminance(rgb1.r, rgb1.g, rgb1.b);
		const lum2 = this.getRelativeLuminance(rgb2.r, rgb2.g, rgb2.b);

		const brightest = Math.max(lum1, lum2);
		const darkest = Math.min(lum1, lum2);

		return (brightest + 0.05) / (darkest + 0.05);
	}

	/**
	 * Check if color combination meets WCAG AA standards
	 */
	static meetsWCAG_AA(
		foreground: string,
		background: string,
		isLargeText = false
	): { passes: boolean; ratio: number | null; required: number } {
		const ratio = this.getContrastRatio(foreground, background);
		const required = isLargeText
			? WCAG_STANDARDS.CONTRAST_RATIO_LARGE
			: WCAG_STANDARDS.CONTRAST_RATIO_NORMAL;

		return {
			passes: ratio !== null && ratio >= required,
			ratio,
			required
		};
	}

	/**
	 * Validate all design token color combinations
	 */
	static validateDesignTokens(): Array<{
		combination: string;
		passes: boolean;
		ratio: number | null;
		required: number;
	}> {
		// These would be extracted from CSS custom properties in a real implementation
		const colorCombinations = [
			{ name: 'primary-text-on-primary-bg', fg: '#212529', bg: '#ffffff', isLarge: false },
			{ name: 'secondary-text-on-primary-bg', fg: '#6c757d', bg: '#ffffff', isLarge: false },
			{ name: 'primary-text-on-secondary-bg', fg: '#212529', bg: '#f8f9fa', isLarge: false },
			{ name: 'primary-button-text', fg: '#ffffff', bg: '#007bff', isLarge: false },
			{ name: 'large-heading', fg: '#212529', bg: '#ffffff', isLarge: true }
		];

		return colorCombinations.map((combo) => ({
			combination: combo.name,
			...this.meetsWCAG_AA(combo.fg, combo.bg, combo.isLarge)
		}));
	}
}

/**
 * Focus Management Utilities
 */
export class FocusManagement {
	private static focusableSelectors = [
		'button',
		'[href]',
		'input:not([disabled])',
		'select:not([disabled])',
		'textarea:not([disabled])',
		'[tabindex]:not([tabindex="-1"])',
		'[contenteditable]'
	].join(',');

	/**
	 * Get all focusable elements within a container
	 */
	static getFocusableElements(container: HTMLElement = document.body): HTMLElement[] {
		return Array.from(container.querySelectorAll(this.focusableSelectors)).filter(
			(el) => el instanceof HTMLElement && this.isVisible(el)
		) as HTMLElement[];
	}

	/**
	 * Check if element is visible and not disabled
	 */
	static isVisible(element: HTMLElement): boolean {
		return (
			!element.hidden &&
			element.offsetParent !== null &&
			window.getComputedStyle(element).display !== 'none'
		);
	}

	/**
	 * Trap focus within a container (for modals, dialogs)
	 */
	static trapFocus(container: HTMLElement): () => void {
		const focusableElements = this.getFocusableElements(container);
		if (focusableElements.length === 0) return () => {};

		const firstElement = focusableElements[0];
		const lastElement = focusableElements[focusableElements.length - 1];

		const handleKeyDown = (event: KeyboardEvent) => {
			if (event.key !== 'Tab') return;

			if (event.shiftKey) {
				// Shift + Tab
				if (document.activeElement === firstElement) {
					event.preventDefault();
					lastElement.focus();
				}
			} else {
				// Tab
				if (document.activeElement === lastElement) {
					event.preventDefault();
					firstElement.focus();
				}
			}
		};

		container.addEventListener('keydown', handleKeyDown);

		// Focus the first element
		firstElement.focus();

		// Return cleanup function
		return () => {
			container.removeEventListener('keydown', handleKeyDown);
		};
	}

	/**
	 * Restore focus to previous element
	 */
	static createFocusRestore() {
		const previousActiveElement = document.activeElement as HTMLElement;

		return () => {
			if (previousActiveElement && this.isVisible(previousActiveElement)) {
				previousActiveElement.focus();
			}
		};
	}

	/**
	 * Move focus to element and scroll into view if needed
	 */
	static focusElement(element: HTMLElement, scrollIntoView = true) {
		element.focus();
		if (scrollIntoView) {
			element.scrollIntoView({
				behavior: 'smooth',
				block: 'nearest'
			});
		}
	}
}

/**
 * Screen Reader Utilities
 */
export class ScreenReader {
	/**
	 * Announce text to screen readers
	 */
	static announce(text: string, priority: 'polite' | 'assertive' = 'polite'): void {
		if (!browser) return;

		const announcer = document.createElement('div');
		announcer.setAttribute('aria-live', priority);
		announcer.setAttribute('aria-atomic', 'true');
		announcer.setAttribute('class', 'sr-only');
		announcer.textContent = text;

		document.body.appendChild(announcer);

		// Remove after announcement
		setTimeout(() => {
			document.body.removeChild(announcer);
		}, 1000);
	}

	/**
	 * Update element's accessible name
	 */
	static updateAccessibleName(element: HTMLElement, name: string): void {
		element.setAttribute('aria-label', name);
	}

	/**
	 * Update element's accessible description
	 */
	static updateAccessibleDescription(element: HTMLElement, description: string): void {
		const descId = `desc-${Math.random().toString(36).substr(2, 9)}`;

		// Create description element
		const descElement = document.createElement('div');
		descElement.id = descId;
		descElement.textContent = description;
		descElement.className = 'sr-only';

		document.body.appendChild(descElement);
		element.setAttribute('aria-describedby', descId);
	}

	/**
	 * Create ARIA live region for dynamic content
	 */
	static createLiveRegion(priority: 'polite' | 'assertive' = 'polite'): HTMLElement {
		const liveRegion = document.createElement('div');
		liveRegion.setAttribute('aria-live', priority);
		liveRegion.setAttribute('aria-atomic', 'true');
		liveRegion.className = 'sr-only';

		document.body.appendChild(liveRegion);
		return liveRegion;
	}
}

/**
 * Motion and Animation Utilities
 */
export class MotionUtils {
	/**
	 * Check if user prefers reduced motion
	 */
	static prefersReducedMotion(): boolean {
		if (!browser) return false;
		return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
	}

	/**
	 * Get appropriate animation duration based on user preference
	 */
	static getAnimationDuration(defaultDuration: number): number {
		return this.prefersReducedMotion()
			? Math.min(defaultDuration, WCAG_STANDARDS.REDUCED_MOTION_DURATION)
			: defaultDuration;
	}

	/**
	 * Conditionally apply animation based on motion preference
	 */
	static conditionalAnimation(
		element: HTMLElement,
		animation: Animation | (() => Animation),
		fallback?: () => void
	): void {
		if (this.prefersReducedMotion()) {
			fallback?.();
		} else {
			const anim = typeof animation === 'function' ? animation() : animation;
			anim.play();
		}
	}
}

/**
 * Touch Target Utilities
 */
export class TouchTargets {
	/**
	 * Validate that interactive elements meet minimum touch target size
	 */
	static validateTouchTargets(container: HTMLElement = document.body): Array<{
		element: HTMLElement;
		width: number;
		height: number;
		meets44px: boolean;
		meets44pxWithSpacing: boolean;
	}> {
		const interactiveElements = container.querySelectorAll(
			'button, [role="button"], a, input, select, textarea, [tabindex]:not([tabindex="-1"])'
		);

		return Array.from(interactiveElements).map((el) => {
			const element = el as HTMLElement;
			const rect = element.getBoundingClientRect();
			const style = window.getComputedStyle(element);

			const width = rect.width;
			const height = rect.height;

			// Check for margin/padding that could provide spacing
			const marginTop = parseInt(style.marginTop, 10) || 0;
			const marginBottom = parseInt(style.marginBottom, 10) || 0;
			const marginLeft = parseInt(style.marginLeft, 10) || 0;
			const marginRight = parseInt(style.marginRight, 10) || 0;

			const effectiveWidth = width + marginLeft + marginRight;
			const effectiveHeight = height + marginTop + marginBottom;

			return {
				element,
				width,
				height,
				meets44px:
					width >= WCAG_STANDARDS.TOUCH_TARGET_MIN && height >= WCAG_STANDARDS.TOUCH_TARGET_MIN,
				meets44pxWithSpacing:
					effectiveWidth >= WCAG_STANDARDS.TOUCH_TARGET_MIN &&
					effectiveHeight >= WCAG_STANDARDS.TOUCH_TARGET_MIN
			};
		});
	}

	/**
	 * Add minimum touch target sizing to an element
	 */
	static ensureMinimumSize(element: HTMLElement): void {
		const rect = element.getBoundingClientRect();
		const style = window.getComputedStyle(element);

		if (rect.width < WCAG_STANDARDS.TOUCH_TARGET_MIN) {
			const currentPadding = parseInt(style.paddingLeft, 10) + parseInt(style.paddingRight, 10);
			const additionalPadding = (WCAG_STANDARDS.TOUCH_TARGET_MIN - rect.width + currentPadding) / 2;
			element.style.paddingLeft = `${additionalPadding}px`;
			element.style.paddingRight = `${additionalPadding}px`;
		}

		if (rect.height < WCAG_STANDARDS.TOUCH_TARGET_MIN) {
			const currentPadding = parseInt(style.paddingTop, 10) + parseInt(style.paddingBottom, 10);
			const additionalPadding =
				(WCAG_STANDARDS.TOUCH_TARGET_MIN - rect.height + currentPadding) / 2;
			element.style.paddingTop = `${additionalPadding}px`;
			element.style.paddingBottom = `${additionalPadding}px`;
		}
	}
}

/**
 * Keyboard Navigation Utilities
 */
export class KeyboardNavigation {
	/**
	 * Handle arrow key navigation for a list of elements
	 */
	static handleArrowNavigation(
		event: KeyboardEvent,
		elements: HTMLElement[],
		currentIndex: number,
		options: {
			horizontal?: boolean;
			vertical?: boolean;
			wrap?: boolean;
		} = {}
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

	/**
	 * Add role-based keyboard navigation to elements
	 */
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
 * Accessibility Testing Utilities
 */
export class A11yTesting {
	/**
	 * Run comprehensive accessibility audit
	 */
	static auditPage(container: HTMLElement = document.body): {
		colorContrast: ReturnType<typeof ColorContrast.validateDesignTokens>;
		touchTargets: ReturnType<typeof TouchTargets.validateTouchTargets>;
		focusableElements: number;
		missingAltText: HTMLImageElement[];
		missingLabels: HTMLElement[];
		headingStructure: Array<{ level: number; text: string; hasContent: boolean }>;
	} {
		// Color contrast validation
		const colorContrast = ColorContrast.validateDesignTokens();

		// Touch target validation
		const touchTargets = TouchTargets.validateTouchTargets(container);

		// Count focusable elements
		const focusableElements = FocusManagement.getFocusableElements(container).length;

		// Find images without alt text
		const images = container.querySelectorAll('img');
		const missingAltText = Array.from(images).filter(
			(img) => !img.hasAttribute('alt') || img.alt.trim() === ''
		) as HTMLImageElement[];

		// Find form controls without labels
		const formControls = container.querySelectorAll('input, select, textarea');
		const missingLabels = Array.from(formControls).filter((control) => {
			const input = control as HTMLInputElement;
			return (
				!input.hasAttribute('aria-label') &&
				!input.hasAttribute('aria-labelledby') &&
				!container.querySelector(`label[for="${input.id}"]`)
			);
		}) as HTMLElement[];

		// Analyze heading structure
		const headings = container.querySelectorAll('h1, h2, h3, h4, h5, h6');
		const headingStructure = Array.from(headings).map((heading) => ({
			level: parseInt(heading.tagName.substring(1), 10),
			text: heading.textContent?.trim() || '',
			hasContent: (heading.textContent?.trim() || '').length > 0
		}));

		return {
			colorContrast,
			touchTargets,
			focusableElements,
			missingAltText,
			missingLabels,
			headingStructure
		};
	}

	/**
	 * Generate accessibility report
	 */
	static generateReport(container?: HTMLElement): string {
		const audit = this.auditPage(container);

		const failedContrast = audit.colorContrast.filter((item) => !item.passes);
		const failedTouchTargets = audit.touchTargets.filter(
			(item) => !item.meets44px && !item.meets44pxWithSpacing
		);

		return `
=== Accessibility Audit Report ===

Color Contrast:
- Total combinations tested: ${audit.colorContrast.length}
- Failed combinations: ${failedContrast.length}
${failedContrast.map((item) => `  - ${item.combination}: ${item.ratio?.toFixed(2)} (needs ${item.required})`).join('\n')}

Touch Targets:
- Total interactive elements: ${audit.touchTargets.length}
- Failed elements: ${failedTouchTargets.length}
${failedTouchTargets.map((item) => `  - Element: ${item.width}x${item.height}px`).join('\n')}

Form Controls:
- Missing labels: ${audit.missingLabels.length}

Images:
- Missing alt text: ${audit.missingAltText.length}

Navigation:
- Focusable elements: ${audit.focusableElements}

Heading Structure:
${audit.headingStructure.map((h) => `${'  '.repeat(h.level - 1)}H${h.level}: ${h.text || '(empty)'}`).join('\n')}
		`.trim();
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
