<!-- web-template/client/src/routes/+page.svelte -->

<script lang="ts">
	import { isAuthenticated, currentUser } from '$lib/stores';
	import { Button, Container, Grid, Flex } from '$lib/components/ui/index.js';
</script>

<svelte:head>
	<title>Web Template - Home</title>
	<meta name="description" content="A modern web application template with authentication" />
</svelte:head>

<main id="main-content" tabindex="-1">
	<Container class="py-16">
		<Flex direction="col" align="center" gap="6" class="text-center">
			<h1 class="text-text-primary text-3xl font-extrabold tracking-tight lg:text-5xl xl:text-6xl">
				Web Application Template
			</h1>
			<p class="text-text-secondary max-w-2xl text-lg leading-relaxed lg:text-xl">
				A modern, secure web application built with SvelteKit and Rust. Features user
				authentication, protected routes, and a beautiful UI with advanced theming.
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
								Welcome back, {$currentUser.email}!
							</p>
						</Flex>
					</div>

					<Flex justify="center" gap="4">
						<Button>
							<a href="/profile" class="text-inherit no-underline"> View Profile </a>
						</Button>
					</Flex>
				</Flex>
			{:else}
				<!-- Guest User View -->
				<Flex justify="center" gap="4" class="flex-wrap">
					<Button>
						<a href="/login" class="text-inherit no-underline"> Sign In </a>
					</Button>
					<Button variant="ghost">
						<a href="/register" class="text-inherit no-underline"> Create Account → </a>
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
				Features
			</h2>
			<Grid cols={{ sm: 1, lg: 2 }} gap="6" class="mx-auto mt-8 max-w-4xl" role="list">
				<article
					class="bg-surface-primary border-border-default rounded-lg border p-6 shadow"
					role="listitem"
				>
					<h3 class="text-text-primary text-base font-semibold">
						<span aria-hidden="true">🔒</span>
						Secure Authentication
					</h3>
					<p class="text-text-secondary mt-2 text-sm">
						JWT-based authentication with Argon2 password hashing and OAuth support
					</p>
				</article>
				<article
					class="bg-surface-primary border-border-default rounded-lg border p-6 shadow"
					role="listitem"
				>
					<h3 class="text-text-primary text-base font-semibold">
						<span aria-hidden="true">⚡</span>
						High Performance
					</h3>
					<p class="text-text-secondary mt-2 text-sm">
						Rust backend with SvelteKit frontend for optimal speed and reliability
					</p>
				</article>
				<article
					class="bg-surface-primary border-border-default rounded-lg border p-6 shadow"
					role="listitem"
				>
					<h3 class="text-text-primary text-base font-semibold">
						<span aria-hidden="true">🎨</span>
						Advanced Theming
					</h3>
					<p class="text-text-secondary mt-2 text-sm">
						Dark/light modes with system preference detection and design tokens
					</p>
				</article>
				<article
					class="bg-surface-primary border-border-default rounded-lg border p-6 shadow"
					role="listitem"
				>
					<h3 class="text-text-primary text-base font-semibold">
						<span aria-hidden="true">🛡️</span>
						Type Safety
					</h3>
					<p class="text-text-secondary mt-2 text-sm">
						Full TypeScript support with strict type checking and modern tooling
					</p>
				</article>
			</Grid>
		</section>
	</Container>
</main>
