<!--
	Navigation Component - Persistent Header Navigation

	Features:
	- Responsive navigation with logo and menu items
	- Theme toggle integration
	- Authentication-aware navigation
	- Mobile-friendly hamburger menu
	- Accessible keyboard navigation
	- Proper ARIA labeling
	- Unified responsive approach
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { isAuthenticated, currentUser } from '$lib/stores';
	import { Container, Flex, Button } from '$lib/components/ui/index.js';
	import ThemeToggle from './ThemeToggle.svelte';
	import LanguageSelector from './LanguageSelector.svelte';
	import { Menu, X, User, Home, ChevronDown, LogOut } from 'lucide-svelte';
	import { cn } from '$lib/utils/index.js';
	import { _ } from 'svelte-i18n';
	import { clickOutside } from '$lib/utils/index.js';

	// Mobile menu state
	let mobileMenuOpen = false;
	// User dropdown menu state
	let userDropdownOpen = false;

	// Navigation items based on authentication state
	$: navigationItems = $isAuthenticated
		? [{ href: '/', label: $_('nav.home'), current: $page.url.pathname === '/', icon: Home }]
		: [
				{ href: '/', label: $_('nav.home'), current: $page.url.pathname === '/', icon: Home },
				{
					href: '/login',
					label: $_('auth.login.submit'),
					current: $page.url.pathname === '/login'
				},
				{
					href: '/register',
					label: $_('auth.register.submit'),
					current: $page.url.pathname === '/register'
				}
			];

	// Close all menus
	const closeAllMenus = () => {
		mobileMenuOpen = false;
		userDropdownOpen = false;
	};

	// Handle logout
	const handleLogout = async () => {
		const { logout } = await import('$lib/services/apiAuth');
		const { goto } = await import('$app/navigation');
		logout();
		closeAllMenus();
		await goto('/login');
	};

	// Handle keyboard navigation
	const handleKeydown = (event: KeyboardEvent) => {
		if (event.key === 'Escape') {
			closeAllMenus();
		}
	};

	// Navigation link classes
	const getLinkClasses = (isCurrent: boolean) =>
		cn(
			'flex items-center gap-2 rounded-md px-3 py-2 font-medium transition-colors duration-fast',
			'focus:outline-none focus:ring-2 focus:ring-amber-400 focus:ring-offset-2',
			isCurrent
				? 'text-primary bg-background-secondary'
				: 'text-text-secondary hover:text-text-primary hover:bg-background-secondary'
		);
</script>

<svelte:window on:keydown={handleKeydown} />

<header class="bg-background-primary sticky top-0 z-50 w-full backdrop-blur-sm">
	<Container>
		<nav aria-label="Main navigation" class="relative">
			<Flex justify="between" align="center" class="h-16">
				<!-- Logo/Brand -->
				<div class="flex-shrink-0">
					<a
						href="/"
						class="text-text-primary duration-fast flex items-center rounded-md text-xl font-bold transition-colors focus:ring-2 focus:ring-amber-400 focus:ring-offset-2 focus:outline-none"
						aria-label="Web Template - Home"
					>
						<span class="text-text-accent mr-2">🚀</span>
						<span class="hidden sm:inline">Web Template</span>
						<span class="sm:hidden">WT</span>
					</a>
				</div>

				<!-- Desktop Navigation -->
				<div class="hidden md:block">
					<Flex align="center" gap="6">
						<!-- Navigation Links -->
						{#each navigationItems as item (item.href)}
							<a
								href={item.href}
								class={getLinkClasses(item.current)}
								aria-current={item.current ? 'page' : undefined}
							>
								{#if item.icon}
									<svelte:component this={item.icon} class="h-4 w-4" aria-hidden="true" />
								{/if}
								<span class="text-sm">{item.label}</span>
							</a>
						{/each}

						<!-- User Menu / Auth Actions -->
						{#if $isAuthenticated && $currentUser}
							<!-- Authenticated User Dropdown -->
							<div class="relative ml-3">
								<Button
									variant="ghost"
									size="sm"
									onclick={() => (userDropdownOpen = !userDropdownOpen)}
									aria-expanded={userDropdownOpen}
									aria-haspopup="menu"
									class="flex items-center gap-2"
								>
									<User class="h-4 w-4" aria-hidden="true" />
									<span class="text-text-secondary hidden max-w-32 truncate text-sm lg:inline">
										{$currentUser.email}
									</span>
									<ChevronDown
										class="duration-fast h-3 w-3 transition-transform {userDropdownOpen
											? 'rotate-180'
											: ''}"
										aria-hidden="true"
									/>
								</Button>

								<!-- User Dropdown Menu -->
								{#if userDropdownOpen}
									<div
										class="border-border-default bg-surface-raised absolute top-full right-0 z-50 mt-2 w-48 rounded-md border shadow-lg"
										role="menu"
										aria-orientation="vertical"
										tabindex="-1"
										use:clickOutside={() => (userDropdownOpen = false)}
									>
										<div class="py-1">
											<button
												type="button"
												class="text-text-primary hover:bg-background-secondary duration-fast flex w-full items-center gap-2 px-4 py-2 text-left text-sm transition-colors focus:ring-2 focus:ring-amber-400 focus:outline-none focus:ring-inset"
												role="menuitem"
												onclick={handleLogout}
											>
												<LogOut class="h-4 w-4" aria-hidden="true" />
												{$_('nav.logout')}
											</button>
										</div>
									</div>
								{/if}
							</div>
						{/if}

						<!-- Language Selector -->
						<LanguageSelector />

						<!-- Theme Toggle -->
						<ThemeToggle />
					</Flex>
				</div>

				<!-- Mobile Actions -->
				<div class="md:hidden">
					<Flex align="center" gap="2">
						<!-- Language Selector (Mobile) -->
						<LanguageSelector />

						<!-- Theme Toggle (Mobile) -->
						<ThemeToggle />

						<!-- Menu Button -->
						<Button
							variant="ghost"
							size="icon"
							onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
							aria-expanded={mobileMenuOpen}
							aria-controls="mobile-menu"
							aria-label={mobileMenuOpen ? 'Close menu' : 'Open menu'}
						>
							{#if mobileMenuOpen}
								<X class="h-5 w-5" aria-hidden="true" />
							{:else}
								<Menu class="h-5 w-5" aria-hidden="true" />
							{/if}
						</Button>
					</Flex>
				</div>
			</Flex>

			<!-- Mobile Navigation Menu -->
			{#if mobileMenuOpen}
				<div
					id="mobile-menu"
					class="bg-background-primary border-border-light absolute top-full right-0 left-0 border-t shadow-lg backdrop-blur-sm md:hidden"
					role="navigation"
					aria-label="Mobile navigation"
					use:clickOutside={() => (mobileMenuOpen = false)}
				>
					<div class="space-y-1 px-4 pt-4 pb-6">
						<!-- Mobile Navigation Links -->
						{#each navigationItems as item (item.href)}
							<a
								href={item.href}
								class={cn(getLinkClasses(item.current), 'text-base')}
								aria-current={item.current ? 'page' : undefined}
								onclick={() => (mobileMenuOpen = false)}
							>
								{#if item.icon}
									<svelte:component this={item.icon} class="h-4 w-4" aria-hidden="true" />
								{/if}
								{item.label}
							</a>
						{/each}

						<!-- Mobile User Menu -->
						{#if $isAuthenticated && $currentUser}
							<div class="border-border-light mt-4 border-t pt-4">
								<div class="flex items-center gap-2 px-3 py-2 text-sm">
									<User class="text-text-secondary h-4 w-4" aria-hidden="true" />
									<span class="text-text-secondary truncate">{$currentUser.email}</span>
								</div>
								<div class="mt-2 space-y-1">
									<button
										type="button"
										class="text-text-secondary hover:bg-background-secondary hover:text-text-primary duration-fast flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-base font-medium transition-colors focus:ring-2 focus:ring-amber-400 focus:ring-offset-2 focus:outline-none"
										onclick={handleLogout}
									>
										<LogOut class="h-4 w-4" aria-hidden="true" />
										{$_('nav.logout')}
									</button>
								</div>
							</div>
						{/if}
					</div>
				</div>
			{/if}
		</nav>
	</Container>
</header>

<!-- Overlay for mobile menu -->
{#if mobileMenuOpen}
	<div class="bg-opacity-25 fixed inset-0 z-40 bg-black md:hidden" aria-hidden="true"></div>
{/if}

<!--
	Usage:
	Import and use in +layout.svelte to show on all pages

	Accessibility Features:
	- Proper ARIA labels and roles
	- Keyboard navigation support
	- Focus management
	- Screen reader friendly
	- Skip links integration
	- Mobile-first responsive design
	- Consistent focus ring styling

	Responsive Behavior:
	- Desktop: Horizontal navigation with full labels
	- Mobile: Hamburger menu with slide-down panel
	- Theme toggle available on both desktop and mobile
	- User info shown contextually
-->
