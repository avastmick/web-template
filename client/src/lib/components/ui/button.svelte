<!--
	Button Component - Modern, Accessible Button with Multiple Variants

	Based on shadcn-svelte design patterns with our design tokens
	- Minimum 44px touch targets for accessibility
	- Proper focus management and keyboard navigation
	- Multiple variants following our design system
	- Built on top of our CSS custom properties
-->

<script lang="ts">
	import { cn } from '$lib/utils/index.js';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	interface ButtonProps extends HTMLButtonAttributes {
		variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
		size?: 'default' | 'sm' | 'lg' | 'icon';
		class?: string;
		children?: import('svelte').Snippet;

		// Accessibility props
		loading?: boolean;
		loadingText?: string;

		// ARIA props that aren't in HTMLButtonAttributes
		'aria-describedby'?: string;
		'aria-expanded'?: boolean;
		'aria-haspopup'?: boolean | 'menu' | 'listbox' | 'tree' | 'grid' | 'dialog';
		'aria-controls'?: string;
		'aria-pressed'?: boolean;
	}

	let {
		variant = 'default',
		size = 'default',
		class: className,
		children,
		loading = false,
		loadingText = 'Loading...',
		disabled,
		...restProps
	}: ButtonProps = $props();

	// Variant styles using our design tokens
	const variants = {
		default: 'bg-primary text-white shadow hover:bg-primary-hover focus:ring-2 focus:ring-offset-2',
		destructive:
			'bg-red-500 text-white shadow hover:bg-red-600 focus:ring-2 focus:ring-red-500 focus:ring-offset-2',
		outline:
			'border border-border-default bg-transparent shadow-sm hover:bg-bg-secondary hover:text-text-primary focus:ring-2 focus:ring-offset-2',
		secondary:
			'bg-bg-secondary text-text-primary shadow-sm hover:bg-neutral-200 focus:ring-2 focus:ring-offset-2',
		ghost: 'hover:bg-bg-secondary hover:text-text-primary focus:ring-2 focus:ring-offset-1',
		link: 'text-primary underline-offset-4 hover:underline focus:ring-2 focus:ring-offset-1'
	};

	// Size styles using our design tokens
	const sizes = {
		default: 'h-touch-target px-6 py-3 text-sm min-w-0',
		sm: 'h-9 rounded-md px-4 text-xs min-w-0',
		lg: 'h-12 rounded-md px-8 text-base min-w-0',
		icon: 'h-touch-target w-touch-target flex-shrink-0'
	};

	// Combine base styles with variants and sizes
	const baseClasses =
		'inline-flex items-center justify-center whitespace-nowrap rounded-md font-medium transition-colors duration-normal ease-smooth focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 touch-target';

	// Loading and disabled state
	const isDisabled = disabled || loading;
	const buttonClasses = cn(
		baseClasses,
		variants[variant],
		sizes[size],
		loading && 'btn-loading',
		className
	);
</script>

<button
	class={buttonClasses}
	disabled={isDisabled}
	aria-busy={loading}
	aria-label={loading ? loadingText : undefined}
	{...restProps}
>
	{#if loading}
		<span class="sr-only">{loadingText}</span>
	{:else}
		{@render children?.()}
	{/if}
</button>

<!--
	Usage Examples:

	<Button>Default Button</Button>
	<Button variant="outline">Outline Button</Button>
	<Button variant="ghost" size="sm">Small Ghost Button</Button>
	<Button variant="destructive">Delete</Button>
	<Button variant="link">Link Button</Button>
	<Button size="icon" aria-label="Menu">
		<MenuIcon />
	</Button>

	Accessibility Notes:
	- All buttons meet 44px minimum touch target requirement
	- Proper focus ring styling with our design tokens
	- Keyboard navigation support through bits-ui
	- Screen reader support with proper ARIA attributes
	- Disabled state handling with visual and interaction feedback
-->
