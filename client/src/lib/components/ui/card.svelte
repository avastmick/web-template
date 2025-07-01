<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';

	interface $$Props extends HTMLAttributes<HTMLDivElement> {
		variant?: 'default' | 'raised' | 'outlined' | 'ghost';
		padding?: 'none' | 'sm' | 'md' | 'lg';
		rounded?: 'none' | 'sm' | 'md' | 'lg' | 'xl';
	}

	export let variant: $$Props['variant'] = 'default';
	export let padding: $$Props['padding'] = 'md';
	export let rounded: $$Props['rounded'] = 'lg';

	const variantStyles = {
		default: 'bg-surface-primary border border-border-default',
		raised: 'bg-surface-raised shadow-md hover:shadow-lg transition-shadow duration-200',
		outlined: 'bg-transparent border-2 border-border-dark',
		ghost: 'bg-surface-secondary'
	};

	const paddingStyles = {
		none: '',
		sm: 'p-3',
		md: 'p-4 sm:p-6',
		lg: 'p-6 sm:p-8'
	};

	const roundedStyles = {
		none: '',
		sm: 'rounded',
		md: 'rounded-md',
		lg: 'rounded-lg',
		xl: 'rounded-xl'
	};

	$: classes = [
		variantStyles[variant || 'default'],
		paddingStyles[padding || 'md'],
		roundedStyles[rounded || 'lg'],
		$$restProps.class || ''
	].join(' ');
</script>

<div {...$$restProps} class={classes}>
	<slot />
</div>
