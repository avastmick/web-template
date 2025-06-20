<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { initTheme } from '$lib/stores/theme.js';
	import { authStore } from '$lib/stores';
	import Navigation from '$lib/components/Navigation.svelte';

	let { children } = $props();

	// Initialize theme system and auth store when component mounts
	onMount(() => {
		initTheme();
		authStore.init();
	});
</script>

<!-- Skip links for keyboard navigation -->
<div class="skip-nav">
	<a href="#main-content" class="skip-link">Skip to main content</a>
	<a href="#primary-navigation" class="skip-link">Skip to navigation</a>
</div>

<!-- Live regions for screen reader announcements -->
<div aria-live="polite" aria-atomic="true" class="sr-only" id="announcements"></div>
<div aria-live="assertive" aria-atomic="true" class="sr-only" id="urgent-announcements"></div>

<!-- Persistent Navigation -->
<Navigation />

<!-- Main Content -->
<div class="bg-bg-primary min-h-screen">
	{@render children()}
</div>
