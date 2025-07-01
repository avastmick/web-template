<script lang="ts">
	import { fade } from 'svelte/transition';
	import { createEventDispatcher } from 'svelte';

	export let variant: 'success' | 'error' | 'warning' | 'info' = 'info';
	export let dismissible = false;
	export let icon = true;
	export let title = '';
	export let description = '';

	const dispatch = createEventDispatcher();

	const variants = {
		success: {
			container: 'bg-status-success-bg border border-status-success/20',
			icon: 'text-status-success',
			title: 'text-status-success',
			description: 'text-status-success/90',
			dismissButton: 'text-status-success hover:text-status-success/80'
		},
		error: {
			container: 'bg-status-error-bg border border-status-error/20',
			icon: 'text-status-error',
			title: 'text-status-error',
			description: 'text-status-error/90',
			dismissButton: 'text-status-error hover:text-status-error/80'
		},
		warning: {
			container: 'bg-status-warning-bg border border-status-warning/20',
			icon: 'text-status-warning',
			title: 'text-status-warning',
			description: 'text-status-warning/90',
			dismissButton: 'text-status-warning hover:text-status-warning/80'
		},
		info: {
			container: 'bg-status-info-bg border border-status-info/20',
			icon: 'text-status-info',
			title: 'text-status-info',
			description: 'text-status-info/90',
			dismissButton: 'text-status-info hover:text-status-info/80'
		}
	};

	function handleDismiss() {
		dispatch('dismiss');
	}

	$: variantStyles = variants[variant];
</script>

<div
	class="flex items-start gap-3 rounded-lg border p-4 {variantStyles.container}"
	role="alert"
	transition:fade={{ duration: 200 }}
>
	{#if icon}
		<svg
			class="h-5 w-5 flex-shrink-0 {variantStyles.icon}"
			viewBox="0 0 20 20"
			fill="currentColor"
			aria-hidden="true"
		>
			{#if variant === 'success'}
				<path
					fill-rule="evenodd"
					d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z"
					clip-rule="evenodd"
				/>
			{:else if variant === 'error'}
				<path
					fill-rule="evenodd"
					d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z"
					clip-rule="evenodd"
				/>
			{:else if variant === 'warning'}
				<path
					fill-rule="evenodd"
					d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 8a1 1 0 100-2 1 1 0 000 2z"
					clip-rule="evenodd"
				/>
			{:else if variant === 'info'}
				<path
					fill-rule="evenodd"
					d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a.75.75 0 000 1.5h.253a.25.25 0 01.244.304l-.459 2.066A1.75 1.75 0 0010.747 15H11a.75.75 0 000-1.5h-.253a.25.25 0 01-.244-.304l.459-2.066A1.75 1.75 0 009.253 9H9z"
					clip-rule="evenodd"
				/>
			{/if}
		</svg>
	{/if}

	<div class="flex-1">
		{#if title}
			<h3 class="text-sm font-medium {variantStyles.title}">{title}</h3>
		{/if}
		{#if description}
			<p class="mt-1 text-sm {variantStyles.description}">{description}</p>
		{/if}
		{#if $$slots.default}
			<div class="mt-2 text-sm {variantStyles.description}">
				<slot />
			</div>
		{/if}
	</div>

	{#if dismissible}
		<button
			type="button"
			class="duration-fast inline-flex rounded-md p-1.5 transition-colors focus:ring-2 focus:ring-amber-500 focus:ring-offset-2 focus:outline-none {variantStyles.dismissButton}"
			on:click={handleDismiss}
			aria-label="Dismiss"
		>
			<svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
				<path
					d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z"
				/>
			</svg>
		</button>
	{/if}
</div>
