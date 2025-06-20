<!--
	Container Component - Responsive Container with Design System Integration

	Features:
	- Responsive width management with max-widths
	- Horizontal padding that adapts to screen size
	- Center alignment with optional full-width variant
	- Integration with our 8px grid system
	- Support for different container sizes (sm, md, lg, xl, 2xl)
	- Mobile-first responsive design
-->

<script lang="ts">
	import { cn } from '$lib/utils/index.js';
	import type { HTMLAttributes } from 'svelte/elements';

	interface ContainerProps extends HTMLAttributes<HTMLDivElement> {
		/**
		 * Container size variant
		 * - 'sm': max-width: 640px
		 * - 'md': max-width: 768px
		 * - 'lg': max-width: 1024px
		 * - 'xl': max-width: 1280px
		 * - '2xl': max-width: 1536px
		 * - 'fluid': no max-width constraint
		 * - 'full': full width (w-full)
		 */
		size?: 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'fluid' | 'full';

		/**
		 * Whether to center the container
		 */
		centered?: boolean;

		/**
		 * Custom padding override
		 */
		padding?: 'none' | 'sm' | 'md' | 'lg' | 'xl';

		/**
		 * Additional CSS classes
		 */
		class?: string;

		/**
		 * Content slot
		 */
		children?: import('svelte').Snippet;
	}

	let {
		size = 'xl',
		centered = true,
		padding = 'md',
		class: className,
		children,
		...restProps
	}: ContainerProps = $props();

	// Container size variants
	const sizeClasses = {
		sm: 'max-w-screen-sm', // 640px
		md: 'max-w-screen-md', // 768px
		lg: 'max-w-screen-lg', // 1024px
		xl: 'max-w-screen-xl', // 1280px
		'2xl': 'max-w-screen-2xl', // 1536px
		fluid: 'max-w-none',
		full: 'w-full'
	};

	// Padding variants using our 8px grid system
	const paddingClasses = {
		none: '',
		sm: 'px-4', // 16px
		md: 'px-6 lg:px-8', // 24px on mobile, 32px on desktop
		lg: 'px-8 lg:px-12', // 32px on mobile, 48px on desktop
		xl: 'px-12 lg:px-16' // 48px on mobile, 64px on desktop
	};

	// Center alignment
	const centerClasses = centered ? 'mx-auto' : '';

	// Combine all classes
	const containerClasses = cn(sizeClasses[size], paddingClasses[padding], centerClasses, className);
</script>

<div class={containerClasses} {...restProps}>
	{@render children?.()}
</div>

<!--
	Usage Examples:

	Basic centered container:
	{@code `<Container>Content here</Container>`}

	Large container with custom padding:
	{@code `<Container size="2xl" padding="lg">Wide content</Container>`}

	Full-width container (no centering):
	{@code `<Container size="full" centered={false}>Full width content</Container>`}

	Fluid container (no max-width):
	{@code `<Container size="fluid">Flexible width content</Container>`}

	Container with custom classes:
	{@code `<Container class="bg-surface-primary border">Styled container</Container>`}

	Responsive Behavior:
	- 'sm' (640px): Good for narrow content like forms
	- 'md' (768px): Good for articles and single-column layouts
	- 'lg' (1024px): Good for most content areas
	- 'xl' (1280px): Good for main site containers (default)
	- '2xl' (1536px): Good for wide layouts on large screens
	- 'fluid': Adapts to parent width
	- 'full': Takes full available width

	Accessibility:
	- Uses semantic div element
	- Maintains reading widths for accessibility
	- Responsive padding prevents content from touching edges
	- Works with screen readers and keyboard navigation
-->
