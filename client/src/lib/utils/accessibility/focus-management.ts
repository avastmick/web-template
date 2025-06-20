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
		firstElement.focus();

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
