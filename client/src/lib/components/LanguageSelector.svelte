<script lang="ts">
	import { locale as svelteI18nLocale } from 'svelte-i18n';
	import { getAllLocales, setLocale } from '$lib/stores/locale';
	import type { SupportedLocale } from '$lib/i18n';

	// Get all available locales
	const locales = getAllLocales();

	// Handle locale change
	function handleLocaleChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		const newLocale = target.value as SupportedLocale;
		setLocale(newLocale);
	}
</script>

<!-- Accessible language selector -->
<div class="inline-block">
	<label for="language-select" class="sr-only">Select Language</label>
	<select
		id="language-select"
		value={$svelteI18nLocale}
		on:change={handleLocaleChange}
		class="border-border-primary hover:border-border-hover text-text-primary
		       min-w-[120px] cursor-pointer rounded-md border bg-transparent px-3
		       py-2 text-sm shadow-none
		       transition-colors duration-200 focus-visible:ring-amber-300 focus-visible:outline-none"
		aria-label="Select language"
	>
		{#each locales as { code, name } (code)}
			<option value={code} class="bg-bg-primary text-text-primary">
				{name}
			</option>
		{/each}
	</select>
</div>
