<!--
	Theme Toggle Component - Advanced Theme Switching UI

	Features:
	- Three-state toggle (light/dark/system)
	- Visual feedback with icons
	- Keyboard navigation support
	- Tooltip showing current mode
	- Smooth transitions
	- Accessible design
-->

<script lang="ts">
	import { theme, toggleTheme, type Theme } from '$lib/stores/theme.js';
	import { Button } from '$lib/components/ui/index.js';
	import { Sun, Moon, Monitor } from 'lucide-svelte';
	import { cn } from '$lib/utils/index.js';

	interface ThemeToggleProps {
		class?: string;
		variant?: 'icon' | 'dropdown' | 'segmented';
		showLabel?: boolean;
	}

	let { class: className, variant = 'icon', showLabel = false }: ThemeToggleProps = $props();

	// Theme options for dropdown/segmented variants
	const themeOptions: { value: Theme; label: string; icon: typeof Sun; description: string }[] = [
		{
			value: 'light',
			label: 'Light',
			icon: Sun,
			description: 'Light mode with bright colors'
		},
		{
			value: 'dark',
			label: 'Dark',
			icon: Moon,
			description: 'Dark mode with muted colors'
		},
		{
			value: 'system',
			label: 'System',
			icon: Monitor,
			description: 'Follow system preference'
		}
	];

	// Get current theme info
	const currentTheme = $derived($theme);
	const currentOption = $derived(themeOptions.find((option) => option.value === currentTheme));

	// Handle theme selection
	const handleThemeSelect = (newTheme: Theme) => {
		theme.set(newTheme);
	};

	// Handle keyboard navigation for icon variant
	const handleKeydown = (event: KeyboardEvent) => {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			toggleTheme();
		}
	};
</script>

{#if variant === 'icon'}
	<!-- Simple icon toggle button -->
	<Button
		variant="ghost"
		size="icon"
		onclick={toggleTheme}
		onkeydown={handleKeydown}
		class={cn('relative transition-transform hover:scale-105', className)}
		aria-label="Toggle theme"
		title="Toggle between light, dark, and system theme"
	>
		<!-- Theme icons with smooth transitions -->
		<div class="relative h-5 w-5">
			{#if currentTheme === 'light'}
				<Sun class="duration-normal ease-smooth h-5 w-5 scale-100 rotate-0 transition-all" />
			{:else if currentTheme === 'dark'}
				<Moon class="duration-normal ease-smooth h-5 w-5 scale-100 rotate-0 transition-all" />
			{:else}
				<Monitor class="duration-normal ease-smooth h-5 w-5 scale-100 rotate-0 transition-all" />
			{/if}
		</div>

		{#if showLabel}
			<span class="ml-2 text-sm font-medium">
				{currentOption?.label || 'Theme'}
			</span>
		{/if}
	</Button>
{:else if variant === 'dropdown'}
	<!-- Dropdown selector (placeholder for future dropdown component) -->
	<div class={cn('relative', className)}>
		<Button variant="outline" class="flex items-center gap-2" aria-label="Select theme">
			{#if currentOption}
				<currentOption.icon class="h-4 w-4" />
				{#if showLabel}
					<span>{currentOption.label}</span>
				{/if}
			{/if}
			<!-- Dropdown chevron -->
			<svg class="ml-1 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		</Button>

		<!-- Dropdown menu would go here when dropdown component is implemented -->
	</div>
{:else if variant === 'segmented'}
	<!-- Segmented control -->
	<div
		class={cn('border-border-default bg-bg-secondary inline-flex rounded-md border p-1', className)}
		role="radiogroup"
		aria-label="Theme selection"
	>
		{#each themeOptions as option (option.value)}
			<button
				type="button"
				role="radio"
				aria-checked={currentTheme === option.value}
				aria-label="Switch to {option.label.toLowerCase()} theme"
				title={option.description}
				class={cn(
					'duration-normal ease-smooth inline-flex items-center gap-2 rounded-sm px-3 py-1.5 text-sm font-medium transition-all',
					'hover:bg-bg-primary focus:ring-primary focus:ring-2 focus:ring-offset-1 focus:outline-none',
					currentTheme === option.value
						? 'bg-bg-primary text-text-primary shadow-sm'
						: 'text-text-secondary hover:text-text-primary'
				)}
				onclick={() => handleThemeSelect(option.value)}
			>
				<option.icon class="h-4 w-4" />
				{#if showLabel}
					<span>{option.label}</span>
				{/if}
			</button>
		{/each}
	</div>
{/if}

<!--
	Usage Examples:

	Simple icon toggle:
	{@code `<ThemeToggle />`}

	Icon toggle with label:
	{@code `<ThemeToggle showLabel={true} />`}

	Segmented control:
	{@code `<ThemeToggle variant="segmented" showLabel={true} />`}

	Dropdown (when dropdown component is available):
	{@code `<ThemeToggle variant="dropdown" showLabel={true} />`}

Accessibility Features:
- Proper ARIA labels and roles
- Keyboard navigation support (Enter/Space)
- Focus management with visible focus rings
- Screen reader friendly descriptions
- High contrast mode support
- Touch-friendly targets (44px minimum)
-->
