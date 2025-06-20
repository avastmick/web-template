/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	// Dark mode configuration
	darkMode: 'class', // Use class-based dark mode

	theme: {
		extend: {
			// Use our design tokens as Tailwind custom properties
			colors: {
				// Semantic colors that map to our CSS custom properties
				primary: 'var(--color-action-primary)',
				'primary-hover': 'var(--color-action-primary-hover)',
				'primary-active': 'var(--color-action-primary-active)',

				// Text colors
				'text-primary': 'var(--color-text-primary)',
				'text-secondary': 'var(--color-text-secondary)',
				'text-muted': 'var(--color-text-muted)',

				// Background colors
				'bg-primary': 'var(--color-background-primary)',
				'bg-secondary': 'var(--color-background-secondary)',

				// Border colors
				'border-default': 'var(--color-border-default)',
				'border-light': 'var(--color-border-light)',
				'border-dark': 'var(--color-border-dark)',

				// Neutral palette
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

				// Blue accent palette
				blue: {
					50: 'var(--color-blue-50)',
					500: 'var(--color-blue-500)',
					600: 'var(--color-blue-600)',
					700: 'var(--color-blue-700)',
					900: 'var(--color-blue-900)'
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

			// Extended grid column support
			gridTemplateColumns: {
				1: 'repeat(1, minmax(0, 1fr))',
				2: 'repeat(2, minmax(0, 1fr))',
				3: 'repeat(3, minmax(0, 1fr))',
				4: 'repeat(4, minmax(0, 1fr))',
				5: 'repeat(5, minmax(0, 1fr))',
				6: 'repeat(6, minmax(0, 1fr))',
				7: 'repeat(7, minmax(0, 1fr))',
				8: 'repeat(8, minmax(0, 1fr))',
				9: 'repeat(9, minmax(0, 1fr))',
				10: 'repeat(10, minmax(0, 1fr))',
				11: 'repeat(11, minmax(0, 1fr))',
				12: 'repeat(12, minmax(0, 1fr))',
				'auto-fit-250': 'repeat(auto-fit, minmax(250px, 1fr))',
				'auto-fill-250': 'repeat(auto-fill, minmax(250px, 1fr))',
				'auto-fit-200': 'repeat(auto-fit, minmax(200px, 1fr))',
				'auto-fill-200': 'repeat(auto-fill, minmax(200px, 1fr))'
			},

			// Container max-widths (matching our breakpoints)
			maxWidth: {
				'screen-sm': '640px',
				'screen-md': '768px',
				'screen-lg': '1024px',
				'screen-xl': '1280px',
				'screen-2xl': '1536px'
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
				lg: 'var(--shadow-lg)'
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

			// Custom utilities
			minHeight: {
				'touch-target': 'var(--touch-target-min)'
			},

			minWidth: {
				'touch-target': 'var(--touch-target-min)'
			}
		}
	},

	// Plugins for enhanced functionality
	plugins: []
};
