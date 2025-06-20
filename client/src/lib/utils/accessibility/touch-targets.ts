/**
 * Touch Target Utilities
 */

import { WCAG_STANDARDS } from './constants.js';

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
