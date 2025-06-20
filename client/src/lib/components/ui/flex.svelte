<!--
	Flex Component - Flexible Layout Container

	Features:
	- CSS Flexbox-based layouts
	- Responsive direction, alignment, and wrap
	- Gap spacing using our 8px grid system
	- Common flex patterns and utilities
	- Mobile-first responsive design
	- Integration with design tokens
-->

<script lang="ts">
	import { cn } from '$lib/utils/index.js';
	import type { HTMLAttributes } from 'svelte/elements';

	interface FlexProps extends HTMLAttributes<HTMLDivElement> {
		/**
		 * Flex direction
		 */
		direction?: 'row' | 'col' | 'row-reverse' | 'col-reverse';

		/**
		 * Responsive flex direction
		 */
		responsiveDirection?: {
			sm?: FlexProps['direction'];
			md?: FlexProps['direction'];
			lg?: FlexProps['direction'];
			xl?: FlexProps['direction'];
			'2xl'?: FlexProps['direction'];
		};

		/**
		 * Align items (cross-axis alignment)
		 */
		align?: 'start' | 'center' | 'end' | 'stretch' | 'baseline';

		/**
		 * Justify content (main-axis alignment)
		 */
		justify?: 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly';

		/**
		 * Flex wrap behavior
		 */
		wrap?: 'nowrap' | 'wrap' | 'wrap-reverse';

		/**
		 * Gap between flex items using our spacing tokens
		 */
		gap?: '1' | '2' | '3' | '4' | '6' | '8' | '12';

		/**
		 * Whether all flex items should grow equally
		 */
		equalGrow?: boolean;

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
		direction = 'row',
		responsiveDirection,
		align = 'start',
		justify = 'start',
		wrap = 'nowrap',
		gap = '4',
		equalGrow = false,
		class: className,
		children,
		...restProps
	}: FlexProps = $props();

	// Direction classes
	const directionClasses = {
		row: 'flex-row',
		col: 'flex-col',
		'row-reverse': 'flex-row-reverse',
		'col-reverse': 'flex-col-reverse'
	};

	// Align items classes
	const alignClasses = {
		start: 'items-start',
		center: 'items-center',
		end: 'items-end',
		stretch: 'items-stretch',
		baseline: 'items-baseline'
	};

	// Justify content classes
	const justifyClasses = {
		start: 'justify-start',
		center: 'justify-center',
		end: 'justify-end',
		between: 'justify-between',
		around: 'justify-around',
		evenly: 'justify-evenly'
	};

	// Wrap classes
	const wrapClasses = {
		nowrap: 'flex-nowrap',
		wrap: 'flex-wrap',
		'wrap-reverse': 'flex-wrap-reverse'
	};

	// Gap classes using our spacing system
	const gapClasses = {
		'1': 'gap-1', // 4px
		'2': 'gap-2', // 8px
		'3': 'gap-3', // 12px
		'4': 'gap-4', // 16px
		'6': 'gap-6', // 24px
		'8': 'gap-8', // 32px
		'12': 'gap-12' // 48px
	};

	// Generate responsive direction classes
	const getResponsiveDirectionClasses = (): string => {
		if (!responsiveDirection) return '';

		const classes: string[] = [];
		if (responsiveDirection.sm) classes.push(`sm:${directionClasses[responsiveDirection.sm]}`);
		if (responsiveDirection.md) classes.push(`md:${directionClasses[responsiveDirection.md]}`);
		if (responsiveDirection.lg) classes.push(`lg:${directionClasses[responsiveDirection.lg]}`);
		if (responsiveDirection.xl) classes.push(`xl:${directionClasses[responsiveDirection.xl]}`);
		if (responsiveDirection['2xl'])
			classes.push(`2xl:${directionClasses[responsiveDirection['2xl']]}`);

		return classes.join(' ');
	};

	// Combine all classes
	const flexClasses = cn(
		'flex',
		directionClasses[direction],
		getResponsiveDirectionClasses(),
		alignClasses[align],
		justifyClasses[justify],
		wrapClasses[wrap],
		gapClasses[gap],
		equalGrow && '*:flex-1',
		className
	);
</script>

<div class={flexClasses} {...restProps}>
	{@render children?.()}
</div>

<!--
	Usage Examples:

	Basic horizontal flex container:
	{@code `<Flex gap="4">
		<div>Item 1</div>
		<div>Item 2</div>
		<div>Item 3</div>
	</Flex>`}

	Centered content:
	{@code `<Flex justify="center" align="center" class="min-h-screen">
		<div>Centered content</div>
	</Flex>`}

	Responsive direction (column on mobile, row on desktop):
	{@code `<Flex direction="col" responsiveDirection={{ lg: 'row' }} gap="6">
		<div>Responsive item 1</div>
		<div>Responsive item 2</div>
	</Flex>`}

	Equal width items:
	{@code `<Flex equalGrow gap="4">
		<div>Equal width 1</div>
		<div>Equal width 2</div>
		<div>Equal width 3</div>
	</Flex>`}

	Space between items:
	{@code `<Flex justify="between" align="center">
		<div>Left item</div>
		<div>Right item</div>
	</Flex>`}

	Common Patterns:

	Header layout:
	{@code `<Flex justify="between" align="center" class="py-4">
		<h1>Logo</h1>
		<nav>Navigation</nav>
	</Flex>`}

	Card layout:
	{@code `<Flex direction="col" gap="4" class="p-6 border rounded-lg">
		<h2>Card Title</h2>
		<p>Card content</p>
		<Flex justify="end" gap="2">
			<Button variant="outline">Cancel</Button>
			<Button>Confirm</Button>
		</Flex>
	</Flex>`}

	Design System Integration:
	- Uses our 8px grid spacing system for gaps
	- Responsive breakpoints match our design tokens
	- Integrates with Tailwind's flex utilities
	- Supports our mobile-first approach

	Accessibility:
	- Maintains logical reading order
	- Works with screen readers
	- Keyboard navigation friendly
	- Respects user's motion preferences
-->
