/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	// Dark mode configuration
	darkMode: 'class', // Use class-based dark mode

	theme: {
		extend: {
			// Use our design tokens as Tailwind custom properties
			colors: {
				// Primary action colors (using CSS variables)
				primary: {
					DEFAULT: 'var(--color-action-primary)',
					hover: 'var(--color-action-primary-hover)',
					active: 'var(--color-action-primary-active)',
					disabled: 'var(--color-action-primary-disabled)'
				},

				// Background colors (simplified naming)
				background: {
					primary: 'var(--color-background-primary)',
					secondary: 'var(--color-background-secondary)',
					tertiary: 'var(--color-background-tertiary)',
					accent: 'var(--color-background-accent)'
				},

				// Surface colors
				surface: {
					primary: 'var(--color-surface-primary)',
					secondary: 'var(--color-surface-secondary)',
					raised: 'var(--color-surface-raised)',
					overlay: 'var(--color-surface-overlay)'
				},

				// Text colors (simplified naming)
				text: {
					primary: 'var(--color-text-primary)',
					secondary: 'var(--color-text-secondary)',
					muted: 'var(--color-text-muted)',
					accent: 'var(--color-text-accent)',
					inverse: 'var(--color-text-inverse)'
				},

				// Border colors
				border: {
					DEFAULT: 'var(--color-border-default)',
					light: 'var(--color-border-light)',
					dark: 'var(--color-border-dark)',
					accent: 'var(--color-border-accent)'
				},

				// Status colors with backgrounds
				status: {
					success: 'var(--color-success)',
					'success-bg': 'var(--color-success-background)',
					warning: 'var(--color-warning)',
					'warning-bg': 'var(--color-warning-background)',
					error: 'var(--color-error)',
					'error-bg': 'var(--color-error-background)',
					info: 'var(--color-info)',
					'info-bg': 'var(--color-info-background)'
				},

				// Focus colors
				focus: {
					ring: 'var(--color-focus-ring)'
				},

				// Neutral palette (keep as-is for granular control)
				neutral: {
					50: 'var(--color-neutral-50)',
					100: 'var(--color-neutral-100)',
					200: 'var(--color-neutral-200)',
					300: 'var(--color-neutral-300)',
					400: 'var(--color-neutral-400)',
					500: 'var(--color-neutral-500)',
					600: 'var(--color-neutral-600)',
					700: 'var(--color-neutral-700)',
					800: 'var(--color-neutral-800)',
					900: 'var(--color-neutral-900)',
					950: 'var(--color-neutral-950)'
				},

				// Blue accent palette (keep for specific use)
				blue: {
					50: 'var(--color-blue-50)',
					500: 'var(--color-blue-500)',
					600: 'var(--color-blue-600)',
					700: 'var(--color-blue-700)',
					900: 'var(--color-blue-900)'
				},

				// Indigo palette for primary theme
				indigo: {
					50: '#eef2ff',
					100: '#e0e7ff',
					200: '#c7d2fe',
					300: '#a5b4fc',
					400: '#818cf8',
					500: '#6366f1',
					600: '#4f46e5',
					700: '#4338ca',
					800: '#3730a3',
					900: '#312e81',
					950: '#1e1b4b'
				},

				// Amber palette for highlights and focus
				amber: {
					50: '#fffbeb',
					100: '#fef3c7',
					200: '#fde68a',
					300: '#fcd34d',
					400: '#fbbf24',
					500: '#f59e0b',
					600: '#d97706',
					700: '#b45309',
					800: '#92400e',
					900: '#78350f',
					950: '#451a03'
				}
			},

			// Typography using our design tokens
			fontSize: {
				xs: 'var(--font-size-xs)',
				sm: 'var(--font-size-sm)',
				base: 'var(--font-size-base)',
				lg: 'var(--font-size-lg)',
				xl: 'var(--font-size-xl)',
				'2xl': 'var(--font-size-2xl)',
				'3xl': 'var(--font-size-3xl)',
				'4xl': 'var(--font-size-4xl)',
				'5xl': 'var(--font-size-5xl)',
				'6xl': 'var(--font-size-6xl)'
			},

			fontWeight: {
				normal: 'var(--font-weight-normal)',
				medium: 'var(--font-weight-medium)',
				semibold: 'var(--font-weight-semibold)',
				bold: 'var(--font-weight-bold)',
				extrabold: 'var(--font-weight-extrabold)'
			},

			lineHeight: {
				tight: 'var(--line-height-tight)',
				snug: 'var(--line-height-snug)',
				normal: 'var(--line-height-normal)',
				relaxed: 'var(--line-height-relaxed)'
			},

			// Spacing using our 8px grid system
			spacing: {
				1: 'var(--space-1)',
				2: 'var(--space-2)',
				3: 'var(--space-3)',
				4: 'var(--space-4)',
				6: 'var(--space-6)',
				8: 'var(--space-8)',
				12: 'var(--space-12)'
			},

			// Container padding
			padding: {
				'container-sm': 'var(--container-padding-sm)',
				'container-md': 'var(--container-padding-md)',
				'container-lg': 'var(--container-padding-lg)',
				'container-xl': 'var(--container-padding-xl)'
			},

			// Border radius
			borderRadius: {
				none: 'var(--radius-none)',
				sm: 'var(--radius-sm)',
				DEFAULT: 'var(--radius-default)',
				md: 'var(--radius-md)',
				lg: 'var(--radius-lg)',
				full: 'var(--radius-full)'
			},

			// Box shadows
			boxShadow: {
				sm: 'var(--shadow-sm)',
				DEFAULT: 'var(--shadow-default)',
				md: 'var(--shadow-md)',
				lg: 'var(--shadow-lg)',
				color: 'var(--shadow-color)',
				'color-strong': 'var(--shadow-color-strong)'
			},

			// Ring styles for focus states
			ringWidth: {
				DEFAULT: 'var(--focus-ring-width)'
			},

			ringOffsetWidth: {
				DEFAULT: 'var(--focus-ring-offset)'
			},

			ringColor: {
				DEFAULT: 'var(--color-focus-ring)'
			},

			// Transitions
			transitionDuration: {
				fast: 'var(--transition-duration-fast)',
				normal: 'var(--transition-duration-normal)',
				slow: 'var(--transition-duration-slow)'
			},

			transitionTimingFunction: {
				smooth: 'var(--transition-easing-smooth)',
				'ease-in': 'var(--transition-easing-ease-in)',
				'ease-out': 'var(--transition-easing-ease-out)'
			},

			// Touch target sizing
			minHeight: {
				'touch-target': 'var(--touch-target-min)'
			},

			minWidth: {
				'touch-target': 'var(--touch-target-min)'
			},

			// Breakpoints (for reference)
			screens: {
				sm: 'var(--breakpoint-sm)',
				md: 'var(--breakpoint-md)',
				lg: 'var(--breakpoint-lg)',
				xl: 'var(--breakpoint-xl)',
				'2xl': 'var(--breakpoint-2xl)'
			}
		}
	},

	// Plugins for enhanced functionality
	plugins: []
};
