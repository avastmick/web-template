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
<div class="bg-background-primary flex h-[calc(100vh-4rem)] overflow-hidden">
	<!-- Conversation Sidebar -->
	<ConversationSidebar />

	<!-- Main Chat Area -->
	<main class="flex flex-1 flex-col overflow-hidden">
		<!-- Clean header -->
		<header
			class="border-border bg-surface-primary flex h-14 flex-shrink-0 items-center justify-between border-b px-4"
		>
			<div class="flex items-center gap-3">
				<!-- Sidebar toggle -->
				<Button
					variant="ghost"
					size="sm"
					onclick={toggleSidebar}
					class="hover:bg-background-secondary h-8 w-8 p-0"
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
				<div class="text-text-secondary text-sm font-medium">
					{$_('chat.header.newChat')}
				</div>
			</div>

			<!-- Streaming indicator -->
			{#if $isStreaming}
				<div class="text-text-muted flex items-center gap-2 text-sm">
					<div class="flex space-x-1">
						<div
							class="bg-text-muted h-2 w-2 animate-bounce rounded-full [animation-delay:-0.3s]"
						></div>
						<div
							class="bg-text-muted h-2 w-2 animate-bounce rounded-full [animation-delay:-0.15s]"
						></div>
						<div class="bg-text-muted h-2 w-2 animate-bounce rounded-full"></div>
					</div>
					<span>{$_('chat.header.streaming')}</span>
				</div>
			{/if}
		</header>

		<!-- Error notification -->
		{#if $error}
			<div class="border-status-error bg-status-error-bg mx-4 mt-4 rounded-lg border p-4">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<svg
							class="text-status-error h-4 w-4"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
							/>
						</svg>
						<p class="text-status-error text-sm font-medium">
							{$_('chat.error.title')}
						</p>
					</div>
					<Button
						variant="ghost"
						size="sm"
						onclick={clearError}
						class="text-status-error hover:bg-status-error-bg h-6 w-6 p-0"
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
				<p class="text-status-error mt-1 text-sm">{$error}</p>
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
								class="bg-status-success-bg mx-auto mb-8 flex h-20 w-20 items-center justify-center rounded-full"
							>
								<svg
									class="text-status-success h-10 w-10"
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
							<h1 class="text-text-primary mb-4 text-3xl font-semibold">
								{$_('chat.welcome.title')}
							</h1>

							<p class="text-text-secondary mb-8 text-lg">
								{$_('chat.welcome.description')}
							</p>

							<!-- Suggestion cards -->
							<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
								{#each [$_('chat.welcome.suggestions.explain'), $_('chat.welcome.suggestions.help'), $_('chat.welcome.suggestions.create')] as suggestion (suggestion)}
									<button
										onclick={() => chatStore.setInputText(suggestion)}
										class="border-border bg-surface-primary hover:bg-surface-secondary rounded-xl border p-4 text-left transition-colors"
									>
										<div class="text-text-primary text-sm font-medium">
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
								class="border-border border-t-primary mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-2"
							></div>
							<p class="text-text-muted text-sm">{$_('chat.loading')}</p>
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
						class="border-border bg-surface-primary hover:bg-surface-secondary h-10 w-10 rounded-full p-0 shadow-lg"
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
			<div class="border-border bg-surface-primary flex-shrink-0 border-t">
				<ChatInput on:send={handleMessageSend} />
			</div>
		</div>
	</main>
</div>
