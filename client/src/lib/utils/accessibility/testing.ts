/**
 * Accessibility Testing Utilities
 */

import { ColorContrast } from './color-contrast.js';
import { TouchTargets } from './touch-targets.js';
import { FocusManagement } from './focus-management.js';

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
