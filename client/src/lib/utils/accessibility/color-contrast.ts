/**
 * Color Contrast Utilities - WCAG 2.1 AA Compliance
 */

import { WCAG_STANDARDS } from './constants.js';

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
