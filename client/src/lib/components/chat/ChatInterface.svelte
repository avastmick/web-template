<!-- Modern chat interface inspired by ChatGPT -->

<script lang="ts">
	import { onDestroy } from 'svelte';
	import { _ } from 'svelte-i18n';
	import { Button } from '$lib/components/ui/index.js';
	import ConversationSidebar from './ConversationSidebar.svelte';
	import MessageBubble from './MessageBubble.svelte';
	import ChatInput from './ChatInput.svelte';
	import {
		chatStore,
		currentConversation,
		isLoading,
		isStreaming,
		error,
		sidebarOpen
	} from '$lib/stores/chatStore.js';

	let messagesContainer: HTMLElement;
	let autoScrollEnabled = true;

	// Scroll to bottom of messages
	function scrollToBottom(smooth = true) {
		if (messagesContainer && autoScrollEnabled) {
			messagesContainer.scrollTo({
				top: messagesContainer.scrollHeight,
				behavior: smooth ? 'smooth' : 'instant'
			});
		}
	}

	// Handle scroll to detect if user scrolled up
	function handleScroll() {
		if (!messagesContainer) return;

		const { scrollTop, scrollHeight, clientHeight } = messagesContainer;
		const isNearBottom = scrollHeight - scrollTop - clientHeight < 100;
		autoScrollEnabled = isNearBottom;
	}

	// Auto-scroll when new messages arrive or streaming updates
	$: if ($currentConversation?.messages || $isStreaming) {
		setTimeout(() => scrollToBottom(), 100);
	}

	// Toggle sidebar
	function toggleSidebar() {
		chatStore.toggleSidebar();
	}

	// Clear error
	function clearError() {
		chatStore.clearError();
	}

	// Handle message send from input component
	function handleMessageSend(event: CustomEvent<{ message: string; files: File[] }>) {
		// The actual sending is handled by the ChatInput component via chatStore
		console.log('Message sent:', event.detail);
	}

	// Cleanup on destroy
	onDestroy(() => {
		chatStore.reset();
	});

	// Responsive sidebar handling
	let windowWidth: number;
	$: if (windowWidth < 1024) {
		// On mobile, close sidebar when a conversation is loaded
		if ($currentConversation && $sidebarOpen) {
			chatStore.setSidebarOpen(false);
		}
	}
</script>

<svelte:window bind:innerWidth={windowWidth} />

<!-- Modern ChatGPT-style layout -->
<div class="flex h-[calc(100vh-4rem)] overflow-hidden bg-white dark:bg-gray-800">
	<!-- Conversation Sidebar -->
	<ConversationSidebar />

	<!-- Main Chat Area -->
	<main class="flex flex-1 flex-col overflow-hidden">
		<!-- Clean header -->
		<header
			class="flex h-14 flex-shrink-0 items-center justify-between border-b border-gray-200 bg-white px-4 dark:border-gray-700 dark:bg-gray-800"
		>
			<div class="flex items-center gap-3">
				<!-- Sidebar toggle -->
				<Button
					variant="ghost"
					size="sm"
					onclick={toggleSidebar}
					class="h-8 w-8 p-0 hover:bg-gray-100 dark:hover:bg-gray-700"
					aria-label={$sidebarOpen ? $_('chat.header.closeSidebar') : $_('chat.header.openSidebar')}
				>
					<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4 6h16M4 12h16M4 18h16"
						/>
					</svg>
				</Button>

				<!-- Model indicator -->
				<div class="text-sm font-medium text-gray-700 dark:text-gray-300">
					{$_('chat.header.newChat')}
				</div>
			</div>

			<!-- Streaming indicator -->
			{#if $isStreaming}
				<div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
					<div class="flex space-x-1">
						<div
							class="h-2 w-2 animate-bounce rounded-full bg-gray-400 [animation-delay:-0.3s]"
						></div>
						<div
							class="h-2 w-2 animate-bounce rounded-full bg-gray-400 [animation-delay:-0.15s]"
						></div>
						<div class="h-2 w-2 animate-bounce rounded-full bg-gray-400"></div>
					</div>
					<span>{$_('chat.header.streaming')}</span>
				</div>
			{/if}
		</header>

		<!-- Error notification -->
		{#if $error}
			<div
				class="mx-4 mt-4 rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-800 dark:bg-red-900/20"
			>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<svg class="h-4 w-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
							/>
						</svg>
						<p class="text-sm font-medium text-red-800 dark:text-red-200">
							{$_('chat.error.title')}
						</p>
					</div>
					<Button
						variant="ghost"
						size="sm"
						onclick={clearError}
						class="h-6 w-6 p-0 text-red-500 hover:bg-red-100 dark:hover:bg-red-800"
						aria-label={$_('chat.error.dismiss')}
					>
						<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M6 18L18 6M6 6l12 12"
							/>
						</svg>
					</Button>
				</div>
				<p class="mt-1 text-sm text-red-700 dark:text-red-300">{$error}</p>
			</div>
		{/if}

		<!-- Messages Area -->
		<div class="flex flex-1 flex-col overflow-hidden">
			<div
				bind:this={messagesContainer}
				onscroll={handleScroll}
				class="flex-1 overflow-y-auto pb-4"
				role="log"
				aria-live="polite"
				aria-label={$_('chat.messages.region')}
			>
				{#if $currentConversation?.messages.length}
					<!-- Messages list -->
					<div class="mx-auto max-w-3xl px-4 py-6">
						{#each $currentConversation.messages as message (message.id)}
							<MessageBubble
								{message}
								isStreaming={$isStreaming &&
									message.role === 'assistant' &&
									message ===
										$currentConversation.messages[$currentConversation.messages.length - 1]}
							/>
						{/each}
					</div>
				{:else if !$isLoading}
					<!-- Modern welcome screen -->
					<div class="flex h-full items-center justify-center px-4">
						<div class="w-full max-w-2xl text-center">
							<!-- ChatGPT-style logo -->
							<div
								class="mx-auto mb-8 flex h-20 w-20 items-center justify-center rounded-full bg-green-100 dark:bg-green-900/20"
							>
								<svg
									class="h-10 w-10 text-green-600 dark:text-green-400"
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
									/>
								</svg>
							</div>

							<!-- Welcome title -->
							<h1 class="mb-4 text-3xl font-semibold text-gray-900 dark:text-white">
								{$_('chat.welcome.title')}
							</h1>

							<p class="mb-8 text-lg text-gray-600 dark:text-gray-300">
								{$_('chat.welcome.description')}
							</p>

							<!-- Suggestion cards -->
							<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
								{#each [$_('chat.welcome.suggestions.explain'), $_('chat.welcome.suggestions.help'), $_('chat.welcome.suggestions.create')] as suggestion (suggestion)}
									<button
										onclick={() => chatStore.setInputText(suggestion)}
										class="rounded-xl border border-gray-200 bg-white p-4 text-left transition-colors hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700"
									>
										<div class="text-sm font-medium text-gray-900 dark:text-white">
											{suggestion}
										</div>
									</button>
								{/each}
							</div>
						</div>
					</div>
				{/if}

				<!-- Loading indicator -->
				{#if $isLoading}
					<div class="flex items-center justify-center py-12">
						<div class="text-center">
							<div
								class="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-2 border-gray-300 border-t-blue-600"
							></div>
							<p class="text-sm text-gray-500 dark:text-gray-400">{$_('chat.loading')}</p>
						</div>
					</div>
				{/if}
			</div>

			<!-- Scroll to bottom button -->
			{#if !autoScrollEnabled}
				<div class="absolute right-6 bottom-20">
					<Button
						variant="outline"
						size="sm"
						class="h-10 w-10 rounded-full border-gray-200 bg-white p-0 shadow-lg hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700"
						onclick={() => scrollToBottom()}
						aria-label={$_('chat.scrollToBottom')}
					>
						<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M19 14l-7 7m0 0l-7-7m7 7V3"
							/>
						</svg>
					</Button>
				</div>
			{/if}

			<!-- Modern Chat Input -->
			<div
				class="flex-shrink-0 border-t border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-800"
			>
				<ChatInput on:send={handleMessageSend} />
			</div>
		</div>
	</main>
</div>
