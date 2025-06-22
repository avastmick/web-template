<!-- web-template/client/src/routes/+page.svelte -->

<script lang="ts">
	import { isAuthenticated, currentUser } from '$lib/stores';
	import { Button, Container, Grid, Flex } from '$lib/components/ui/index.js';
	import { _ } from 'svelte-i18n';
</script>

<svelte:head>
	<title>{$_('home.title')}</title>
	<meta name="description" content={$_('home.description')} />
</svelte:head>

<main id="main-content" tabindex="-1">
	<Container class="py-16">
		<Flex direction="col" align="center" gap="6" class="text-center">
			<h1 class="text-text-primary text-3xl font-extrabold tracking-tight lg:text-5xl xl:text-6xl">
				{$_('home.title')}
			</h1>
			<p class="text-text-secondary max-w-2xl text-lg leading-relaxed lg:text-xl">
				{$_('home.description')}
			</p>

			{#if $isAuthenticated && $currentUser}
				<!-- Authenticated User View -->
				<Flex direction="col" gap="6" class="w-full max-w-md">
					<div class="bg-color-success-background border-color-success rounded-md border p-4">
						<Flex align="center" gap="3">
							<div class="flex-shrink-0">
								<svg
									class="text-color-success h-5 w-5"
									viewBox="0 0 20 20"
									fill="currentColor"
									aria-hidden="true"
								>
									<path
										fill-rule="evenodd"
										d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.236 4.53L7.73 10.16a.75.75 0 00-1.06 1.06l2.25 2.25a.75.75 0 001.14-.094l3.75-5.25z"
										clip-rule="evenodd"
									/>
								</svg>
							</div>
							<p class="text-color-success text-sm font-medium">
								{$_('home.welcomeBack', { values: { email: $currentUser.email } })}
							</p>
						</Flex>
					</div>

					<Flex justify="center" gap="4">
						<Button>
							<a href="/profile" class="text-inherit no-underline">{$_('home.viewProfile')}</a>
						</Button>
					</Flex>
				</Flex>
			{:else}
				<!-- Guest User View -->
				<Flex justify="center" gap="4" class="flex-wrap">
					<Button>
						<a href="/login" class="text-inherit no-underline">{$_('auth.login.submit')}</a>
					</Button>
					<Button variant="ghost">
						<a href="/register" class="text-inherit no-underline">{$_('home.createAccount')}</a>
					</Button>
				</Flex>
			{/if}
		</Flex>

		<!-- Features Section -->
		<section class="mt-16 w-full" aria-labelledby="features-heading">
			<h2
				id="features-heading"
				class="text-text-primary text-center text-2xl font-bold lg:text-3xl"
			>
				{$_('home.features')}
			</h2>
			<Grid cols={{ sm: 1, lg: 2 }} gap="6" class="mx-auto mt-8 max-w-4xl" role="list">
				<article
					class="bg-surface-primary border-border-default rounded-lg border p-6 shadow"
					role="listitem"
				>
					<h3 class="text-text-primary text-base font-semibold">
						<span aria-hidden="true">🔒</span>
						{$_('home.feature.auth.title')}
					</h3>
					<p class="text-text-secondary mt-2 text-sm">
						{$_('home.feature.auth.description')}
					</p>
				</article>
				<article
					class="bg-surface-primary border-border-default rounded-lg border p-6 shadow"
					role="listitem"
				>
					<h3 class="text-text-primary text-base font-semibold">
						<span aria-hidden="true">⚡</span>
						{$_('home.feature.performance.title')}
					</h3>
					<p class="text-text-secondary mt-2 text-sm">
						{$_('home.feature.performance.description')}
					</p>
				</article>
				<article
					class="bg-surface-primary border-border-default rounded-lg border p-6 shadow"
					role="listitem"
				>
					<h3 class="text-text-primary text-base font-semibold">
						<span aria-hidden="true">🎨</span>
						{$_('home.feature.theming.title')}
					</h3>
					<p class="text-text-secondary mt-2 text-sm">
						{$_('home.feature.theming.description')}
					</p>
				</article>
				<article
					class="bg-surface-primary border-border-default rounded-lg border p-6 shadow"
					role="listitem"
				>
					<h3 class="text-text-primary text-base font-semibold">
						<span aria-hidden="true">🛡️</span>
						{$_('home.feature.types.title')}
					</h3>
					<p class="text-text-secondary mt-2 text-sm">
						{$_('home.feature.types.description')}
					</p>
				</article>
			</Grid>
		</section>
	</Container>
</main>
