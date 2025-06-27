<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { initTheme } from '$lib/stores/theme.js';
	import { authStore } from '$lib/stores';
	import { initializeI18n, dir } from '$lib/i18n';
	import { waitLocale, _ } from 'svelte-i18n';
	import { locale } from '$lib/stores/locale';
	import Navigation from '$lib/components/Navigation.svelte';
	import { afterNavigate } from '$app/navigation';

	let { children } = $props();

	// Initialize i18n immediately
	initializeI18n();

	// Initialize theme system and auth store when component mounts
	onMount(() => {
		initTheme();
		authStore.init();
	});

	// Ensure proper cleanup after navigation
	afterNavigate(() => {
		// Force scroll to top on navigation
		window.scrollTo(0, 0);
		// Clear any lingering focus states
		if (document.activeElement instanceof HTMLElement) {
			document.activeElement.blur();
		}
	});

	// Apply direction and language to document
	$effect(() => {
		if (typeof document !== 'undefined') {
			document.documentElement.dir = $dir;
			document.documentElement.lang = $locale;
		}
	});
</script>

{#await waitLocale()}
	<!-- Loading state while translations are being loaded -->
	<div class="bg-bg-primary flex min-h-screen items-center justify-center">
		<div class="text-center">
			<div
				class="border-accent-primary mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-b-2"
			></div>
			<p class="text-text-secondary">Loading...</p>
		</div>
	</div>
{:then}
	<!-- Skip links for keyboard navigation -->
	<div class="skip-nav">
		<a href="#main-content" class="skip-link">{$_('accessibility.skipToMain')}</a>
		<a href="#primary-navigation" class="skip-link">{$_('accessibility.skipToNav')}</a>
	</div>

	<!-- Live regions for screen reader announcements -->
	<div aria-live="polite" aria-atomic="true" class="sr-only" id="announcements"></div>
	<div aria-live="assertive" aria-atomic="true" class="sr-only" id="urgent-announcements"></div>

	<!-- Persistent Navigation -->
	<Navigation />

	<!-- Main Content -->
	<main id="main-content">
		{@render children()}
	</main>
{/await}
