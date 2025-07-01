<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import Input from './input.svelte';

	interface $$Props extends HTMLInputAttributes {
		label: string;
		error?: string;
		hint?: string;
		required?: boolean;
		inputClass?: string;
	}

	export let label: $$Props['label'];
	export let error: $$Props['error'] = undefined;
	export let hint: $$Props['hint'] = undefined;
	export let required: $$Props['required'] = false;
	export let inputClass: $$Props['inputClass'] = '';
	export let id: $$Props['id'] = crypto.randomUUID();
	export let value: $$Props['value'] = '';
</script>

<div class="w-full space-y-1.5">
	<label for={id} class="text-text-secondary block text-sm font-medium">
		{label}
		{#if required}
			<span class="text-status-error" aria-label="required">*</span>
		{/if}
	</label>

	<Input
		{id}
		{...$$restProps}
		bind:value
		class={inputClass}
		aria-invalid={!!error}
		aria-describedby={error ? `${id}-error` : hint ? `${id}-hint` : undefined}
	/>

	{#if error}
		<p id="{id}-error" class="text-status-error text-sm" role="alert">
			{error}
		</p>
	{:else if hint}
		<p id="{id}-hint" class="text-text-muted text-sm">
			{hint}
		</p>
	{/if}
</div>
