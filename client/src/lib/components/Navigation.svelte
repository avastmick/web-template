<!--
	Navigation Component - Persistent Header Navigation

	Features:
	- Responsive navigation with logo and menu items
	- Theme toggle integration
	- Authentication-aware navigation
	- Mobile-friendly hamburger menu
	- Accessible keyboard navigation
	- Proper ARIA labeling
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { isAuthenticated, currentUser } from '$lib/stores';
	import { Container, Flex, Button } from '$lib/components/ui/index.js';
	import ThemeToggle from './ThemeToggle.svelte';
	import { Menu, X, User, Home, ChevronDown, LogOut } from 'lucide-svelte';
	import { cn } from '$lib/utils/index.js';

	// Mobile menu state
	let mobileMenuOpen = false;
	// User dropdown menu state
	let userDropdownOpen = false;

	// Navigation items based on authentication state
	$: navigationItems = $isAuthenticated
		? [{ href: '/', label: 'Home', current: $page.url.pathname === '/', icon: Home }]
		: [
				{ href: '/', label: 'Home', current: $page.url.pathname === '/', icon: Home },
				{ href: '/login', label: 'Sign In', current: $page.url.pathname === '/login' },
				{ href: '/register', label: 'Register', current: $page.url.pathname === '/register' }
			];

	// Close mobile menu when clicking outside or on navigation
	const closeMobileMenu = () => {
		mobileMenuOpen = false;
	};

	// Close user dropdown
	const closeUserDropdown = () => {
		userDropdownOpen = false;
	};

	// Handle logout
	const handleLogout = async () => {
		const { logout } = await import('$lib/services/apiAuth');
		const { goto } = await import('$app/navigation');
		logout();
		closeUserDropdown();
		await goto('/login');
	};

	// Handle keyboard navigation for mobile menu
	const handleMenuKeydown = (event: KeyboardEvent) => {
		if (event.key === 'Escape') {
			closeMobileMenu();
		}
	};
</script>

<header
	class="bg-bg-primary border-border-default sticky top-0 z-50 w-full border-b backdrop-blur-sm"
>
	<Container>
		<nav aria-label="Main navigation" class="relative">
			<Flex justify="between" align="center" class="h-16">
				<!-- Logo/Brand -->
				<div class="flex-shrink-0">
					<a
						href="/"
						class="text-text-primary focus-visible-ring flex items-center text-xl font-bold"
						aria-label="Web Template - Home"
					>
						<span class="text-primary mr-2">🚀</span>
						<span class="hidden sm:inline">Web Template</span>
						<span class="sm:hidden">WT</span>
					</a>
				</div>

				<!-- Desktop Navigation -->
				<div class="hidden md:block">
					<Flex align="center" gap="8">
						<!-- Navigation Links -->
						<Flex align="center" gap="6">
							{#each navigationItems as item (item.href)}
								<a
									href={item.href}
									class={cn(
										'focus-visible-ring flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
										item.current
											? 'text-primary bg-bg-secondary'
											: 'text-text-secondary hover:text-text-primary hover:bg-bg-secondary'
									)}
									aria-current={item.current ? 'page' : undefined}
								>
									{#if item.icon}
										<svelte:component this={item.icon} class="h-4 w-4" aria-hidden="true" />
									{/if}
									{item.label}
								</a>
							{/each}
						</Flex>

						<!-- User Menu / Auth Actions -->
						<Flex align="center" gap="4">
							{#if $isAuthenticated && $currentUser}
								<!-- Authenticated User Dropdown -->
								<div class="relative">
									<Button
										variant="ghost"
										size="sm"
										onclick={() => (userDropdownOpen = !userDropdownOpen)}
										aria-expanded={userDropdownOpen}
										aria-haspopup="menu"
										class="flex items-center gap-2"
									>
										<User class="h-4 w-4" aria-hidden="true" />
										<span class="text-text-secondary hidden max-w-32 truncate lg:inline">
											{$currentUser.email}
										</span>
										<ChevronDown class="h-3 w-3" aria-hidden="true" />
									</Button>

									<!-- User Dropdown Menu -->
									{#if userDropdownOpen}
										<div
											class="border-border-default absolute top-full right-0 z-50 mt-2 w-48 rounded-md border shadow-lg"
											style="background-color: var(--color-surface-raised);"
											role="menu"
											aria-orientation="vertical"
											tabindex="-1"
										>
											<div class="py-1">
												<a
													href="/profile"
													class="text-text-primary hover:bg-bg-tertiary focus-visible-ring flex items-center gap-2 px-4 py-2 text-sm transition-colors"
													role="menuitem"
													onclick={closeUserDropdown}
												>
													<User class="h-4 w-4" aria-hidden="true" />
													Profile
												</a>
												<button
													type="button"
													class="text-text-primary hover:bg-bg-tertiary focus-visible-ring flex w-full items-center gap-2 px-4 py-2 text-left text-sm transition-colors"
													role="menuitem"
													onclick={handleLogout}
												>
													<LogOut class="h-4 w-4" aria-hidden="true" />
													Logout
												</button>
											</div>
										</div>
									{/if}
								</div>
							{/if}

							<!-- Theme Toggle -->
							<ThemeToggle />
						</Flex>
					</Flex>
				</div>

				<!-- Mobile Menu Button -->
				<div class="md:hidden">
					<Flex align="center" gap="2">
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
							class="md:hidden"
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
				<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
				<div
					id="mobile-menu"
					class="bg-bg-primary border-border-default absolute top-full right-0 left-0 border-b shadow-lg backdrop-blur-sm md:hidden"
					role="navigation"
					aria-label="Mobile navigation"
					tabindex="-1"
					onkeydown={handleMenuKeydown}
				>
					<div class="space-y-1 px-4 pt-4 pb-6">
						<!-- Mobile Navigation Links -->
						{#each navigationItems as item (item.href)}
							<a
								href={item.href}
								class={cn(
									'focus-visible-ring flex items-center gap-2 rounded-md px-3 py-2 text-base font-medium transition-colors',
									item.current
										? 'text-primary bg-bg-secondary'
										: 'text-text-secondary hover:text-text-primary hover:bg-bg-secondary'
								)}
								aria-current={item.current ? 'page' : undefined}
								onclick={closeMobileMenu}
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
									<User class="h-4 w-4" aria-hidden="true" />
									<span class="text-text-secondary truncate">{$currentUser.email}</span>
								</div>
								<div class="mt-2 space-y-1">
									<a
										href="/profile"
										class="focus-visible-ring text-text-secondary hover:bg-bg-secondary hover:text-text-primary flex items-center gap-2 rounded-md px-3 py-2 text-base font-medium transition-colors"
										onclick={closeMobileMenu}
									>
										<User class="h-4 w-4" aria-hidden="true" />
										Profile
									</a>
									<button
										type="button"
										class="focus-visible-ring text-text-secondary hover:bg-bg-secondary hover:text-text-primary flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-base font-medium transition-colors"
										onclick={handleLogout}
									>
										<LogOut class="h-4 w-4" aria-hidden="true" />
										Logout
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

<!-- Click outside to close mobile menu -->
{#if mobileMenuOpen}
	<div
		class="bg-opacity-25 fixed inset-0 z-40 bg-black md:hidden"
		onclick={closeMobileMenu}
		aria-hidden="true"
	></div>
{/if}

<!-- Click outside to close user dropdown -->
{#if userDropdownOpen}
	<div class="fixed inset-0 z-40" onclick={closeUserDropdown} aria-hidden="true"></div>
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

	Responsive Behavior:
	- Desktop: Horizontal navigation with full labels
	- Mobile: Hamburger menu with overlay
	- Theme toggle available on both desktop and mobile
	- User info shown contextually
-->
