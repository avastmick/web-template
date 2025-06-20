<!--
	Grid Component - Responsive CSS Grid System

	Features:
	- CSS Grid-based responsive layouts
	- Mobile-first responsive design
	- Flexible column configurations
	- Gap spacing using our 8px grid system
	- Support for different grid patterns
	- Auto-fit and auto-fill options
	- Integration with design tokens
-->

<script lang="ts">
	import { cn } from '$lib/utils/index.js';
	import type { HTMLAttributes } from 'svelte/elements';

	interface GridProps extends HTMLAttributes<HTMLDivElement> {
		/**
		 * Number of columns for different breakpoints
		 * Can be a number for consistent columns or object for responsive
		 */
		cols?:
			| number
			| {
					sm?: number;
					md?: number;
					lg?: number;
					xl?: number;
					'2xl'?: number;
			  };

		/**
		 * Gap between grid items using our spacing tokens
		 */
		gap?: '1' | '2' | '3' | '4' | '6' | '8' | '12';

		/**
		 * Minimum width for auto-fit/auto-fill columns
		 */
		minItemWidth?: string;

		/**
		 * Grid behavior
		 * - 'fit': auto-fit (columns collapse when items can't fit)
		 * - 'fill': auto-fill (maintains column count, creates empty columns)
		 * - 'fixed': fixed column count
		 */
		behavior?: 'fit' | 'fill' | 'fixed';

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
		cols = 1,
		gap = '4',
		minItemWidth = '250px',
		behavior = 'fixed',
		class: className,
		children,
		...restProps
	}: GridProps = $props();

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

	// Generate responsive column classes
	const getColumnClasses = (): string => {
		if (typeof cols === 'number') {
			// Simple number - apply to all breakpoints
			return `grid-cols-${cols}`;
		}

		// Responsive object
		const classes: string[] = [];
		if (cols.sm) classes.push(`sm:grid-cols-${cols.sm}`);
		if (cols.md) classes.push(`md:grid-cols-${cols.md}`);
		if (cols.lg) classes.push(`lg:grid-cols-${cols.lg}`);
		if (cols.xl) classes.push(`xl:grid-cols-${cols.xl}`);
		if (cols['2xl']) classes.push(`2xl:grid-cols-${cols['2xl']}`);

		return classes.join(' ');
	};

	// Generate grid template columns for auto-fit/auto-fill
	const getAutoGridStyle = (): string => {
		if (behavior === 'fit') {
			return `grid-template-columns: repeat(auto-fit, minmax(${minItemWidth}, 1fr));`;
		}
		if (behavior === 'fill') {
			return `grid-template-columns: repeat(auto-fill, minmax(${minItemWidth}, 1fr));`;
		}
		return '';
	};

	// Combine all classes
	const gridClasses = cn(
		'grid',
		behavior === 'fixed' ? getColumnClasses() : '',
		gapClasses[gap],
		className
	);

	const gridStyle = behavior !== 'fixed' ? getAutoGridStyle() : '';
</script>

<div class={gridClasses} style={gridStyle} {...restProps}>
	{@render children?.()}
</div>

<!--
	Usage Examples:

	Simple 3-column grid:
	{@code `<Grid cols={3} gap="4">
		<div>Item 1</div>
		<div>Item 2</div>
		<div>Item 3</div>
	</Grid>`}

	Responsive grid (1 column on mobile, 2 on tablet, 3 on desktop):
	{@code `<Grid cols={{ sm: 1, md: 2, lg: 3 }} gap="6">
		<div>Responsive item 1</div>
		<div>Responsive item 2</div>
		<div>Responsive item 3</div>
	</Grid>`}

	Auto-fit grid (columns adjust based on content):
	{@code `<Grid behavior="fit" minItemWidth="200px" gap="4">
		<div>Auto item 1</div>
		<div>Auto item 2</div>
		<div>Auto item 3</div>
	</Grid>`}

	Auto-fill grid (maintains column count):
	{@code `<Grid behavior="fill" minItemWidth="150px" gap="2">
		<div>Fill item 1</div>
		<div>Fill item 2</div>
	</Grid>`}

	Design System Integration:
	- Uses our 8px grid spacing system for gaps
	- Responsive breakpoints match our design tokens
	- Integrates with Tailwind's grid utilities
	- Supports our mobile-first approach

	Responsive Behavior:
	- Mobile-first: starts with single column by default
	- Breakpoints: sm (640px), md (768px), lg (1024px), xl (1280px), 2xl (1536px)
	- Auto-fit: columns collapse when content doesn't fit
	- Auto-fill: maintains column structure with empty space

	Accessibility:
	- Uses semantic grid layout
	- Maintains logical reading order
	- Works with screen readers
	- Respects user's motion preferences
-->
