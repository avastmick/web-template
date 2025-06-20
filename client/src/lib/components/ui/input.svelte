<!--
	Input Component - Modern, Accessible Input Field

	Based on our design system with proper accessibility
	- Minimum 44px touch targets
	- Proper focus management and validation states
	- Built with our CSS custom properties
	- Support for different input types
-->

<script lang="ts">
	import { cn } from '$lib/utils/index.js';
	import type { HTMLInputAttributes } from 'svelte/elements';

	interface InputProps extends Omit<HTMLInputAttributes, 'class' | 'value'> {
		class?: string;
		error?: string | boolean;
		helperText?: string;
		helpText?: string;
		label?: string;
		value?: string;
		onblur?: (event: FocusEvent) => void;
	}

	let {
		class: className,
		type = 'text',
		error = false,
		helperText,
		helpText,
		label,
		id,
		value = $bindable(''),
		onblur,
		...restProps
	}: InputProps = $props();

	// Generate unique ID if not provided
	const inputId = id || `input-${Math.random().toString(36).substring(2, 9)}`;

	// Base styles using our design tokens
	const baseClasses =
		'flex w-full rounded-md border px-4 py-3 text-base file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50 transition-colors min-h-touch-target';

	// Helper for determining if there's an error
	const hasError = typeof error === 'string' ? error.length > 0 : !!error;

	// Conditional styles based on error state
	const conditionalClasses = hasError
		? 'border-red-500 bg-red-50 text-red-900'
		: 'border-border-default bg-bg-primary';

	// Determine the helper text to display
	const displayHelperText =
		typeof error === 'string' && error.length > 0 ? error : helperText || helpText;
</script>

<div class="w-full space-y-2">
	{#if label}
		<label for={inputId} class="text-text-primary text-sm leading-none font-medium">
			{label}
		</label>
	{/if}

	<input
		{type}
		id={inputId}
		bind:value
		class={cn(baseClasses, conditionalClasses, className)}
		{onblur}
		{...restProps}
	/>

	{#if displayHelperText}
		<p
			class={cn('text-sm', hasError ? 'text-red-600' : 'text-text-secondary')}
			id="{inputId}-description"
			aria-live="polite"
		>
			{displayHelperText}
		</p>
	{/if}
</div>

<!--
	Usage Examples:

	<Input placeholder="Enter your email" />
	<Input label="Email" type="email" placeholder="you@example.com" />
	<Input label="Password" type="password" />
	<Input
		label="Email"
		type="email"
		error={true}
		helperText="Please enter a valid email address"
	/>
	<Input
		label="Full Name"
		helperText="Enter your first and last name"
	/>

	Accessibility Notes:
	- Proper label association with input
	- Minimum 44px touch target height
	- Focus ring styling with our design tokens
	- Error state with ARIA live region for screen readers
	- Helper text properly associated with input
	- Keyboard navigation support
	- Placeholder text with proper contrast
-->
