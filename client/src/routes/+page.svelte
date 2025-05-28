<!-- web-template/client/src/routes/+page.svelte -->

<script lang="ts">
	import { onMount } from 'svelte';
	import { authStore, isAuthenticated, currentUser } from '$lib/stores';

	onMount(() => {
		// Initialize auth store on app load
		authStore.init();
	});
</script>

<svelte:head>
	<title>Web Template - Home</title>
	<meta name="description" content="A modern web application template with authentication" />
</svelte:head>

<div class="min-h-screen bg-gray-50">
	<div class="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:px-8">
		<div class="text-center">
			<h1 class="text-4xl font-bold tracking-tight text-gray-900 sm:text-6xl">
				Web Application Template
			</h1>
			<p class="mx-auto mt-6 max-w-2xl text-lg leading-8 text-gray-600">
				A modern, secure web application built with SvelteKit and Rust. Features user
				authentication, protected routes, and a beautiful UI.
			</p>

			{#if $isAuthenticated && $currentUser}
				<!-- Authenticated User View -->
				<div class="mt-10">
					<div class="rounded-md bg-green-50 p-4">
						<div class="flex items-center justify-center">
							<div class="flex-shrink-0">
								<svg
									class="h-5 w-5 text-green-400"
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
							<div class="ml-3">
								<p class="text-sm font-medium text-green-800">
									Welcome back, {$currentUser.email}!
								</p>
							</div>
						</div>
					</div>

					<div class="mt-10 flex items-center justify-center gap-x-6">
						<a
							href="/profile"
							class="rounded-md bg-indigo-600 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
						>
							View Profile
						</a>
					</div>
				</div>
			{:else}
				<!-- Guest User View -->
				<div class="mt-10 flex items-center justify-center gap-x-6">
					<a
						href="/login"
						class="rounded-md bg-indigo-600 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
					>
						Sign In
					</a>
					<a
						href="/register"
						class="text-sm leading-6 font-semibold text-gray-900 hover:text-indigo-600"
					>
						Create Account <span aria-hidden="true">→</span>
					</a>
				</div>
			{/if}

			<!-- Features Section -->
			<div class="mt-16">
				<h2 class="text-2xl font-bold text-gray-900">Features</h2>
				<div class="mx-auto mt-8 max-w-2xl">
					<dl class="grid grid-cols-1 gap-6 sm:grid-cols-2">
						<div class="rounded-lg bg-white p-6 shadow">
							<dt class="text-base font-semibold text-gray-900">🔒 Secure Authentication</dt>
							<dd class="mt-2 text-sm text-gray-600">
								JWT-based authentication with Argon2 password hashing
							</dd>
						</div>
						<div class="rounded-lg bg-white p-6 shadow">
							<dt class="text-base font-semibold text-gray-900">⚡ High Performance</dt>
							<dd class="mt-2 text-sm text-gray-600">
								Rust backend with SvelteKit frontend for optimal speed
							</dd>
						</div>
						<div class="rounded-lg bg-white p-6 shadow">
							<dt class="text-base font-semibold text-gray-900">🎨 Beautiful UI</dt>
							<dd class="mt-2 text-sm text-gray-600">
								Tailwind CSS with responsive design and accessibility
							</dd>
						</div>
						<div class="rounded-lg bg-white p-6 shadow">
							<dt class="text-base font-semibold text-gray-900">🛡️ Type Safety</dt>
							<dd class="mt-2 text-sm text-gray-600">
								Full TypeScript support with strict type checking
							</dd>
						</div>
					</dl>
				</div>
			</div>
		</div>
	</div>
</div>
