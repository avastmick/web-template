<!-- Modern conversation sidebar inspired by ChatGPT -->

<script lang="ts">
	import { onMount } from 'svelte';
	import { _ } from 'svelte-i18n';
	import { Button } from '$lib/components/ui/index.js';
	import {
		chatStore,
		conversations,
		currentConversation,
		sidebarOpen,
		isLoading
	} from '$lib/stores/chatStore.js';

	// Load conversations on mount
	onMount(() => {
		chatStore.loadConversations();
	});

	// Start new conversation
	function startNewConversation() {
		chatStore.startNewConversation();
	}

	// Load existing conversation
	function loadConversation(conversationId: string) {
		chatStore.loadConversation(conversationId);
	}

	// Archive conversation
	function archiveConversation(conversationId: string, event: MouseEvent) {
		event.stopPropagation();
		chatStore.archiveConversation(conversationId);
	}

	// Delete conversation
	function deleteConversation(conversationId: string, event: MouseEvent) {
		event.stopPropagation();
		if (confirm($_('chat.sidebar.confirmDelete'))) {
			chatStore.deleteConversation(conversationId);
		}
	}

	// Format conversation title
	function getConversationTitle(conversation: {
		title?: string;
		last_message?: { content: string };
	}): string {
		if (conversation.title) {
			return conversation.title as string;
		}

		// Generate title from first message or use default
		if (conversation.last_message?.content) {
			const content = conversation.last_message.content.slice(0, 50);
			return content.length < conversation.last_message.content.length ? content + '...' : content;
		}

		return $_('chat.sidebar.untitledConversation');
	}

	// Format relative time
	function getRelativeTime(dateString: string): string {
		const date = new Date(dateString);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffMins = Math.floor(diffMs / (1000 * 60));
		const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

		if (diffMins < 1) return $_('chat.sidebar.time.now');
		if (diffMins < 60) return $_('chat.sidebar.time.minutes', { values: { count: diffMins } });
		if (diffHours < 24) return $_('chat.sidebar.time.hours', { values: { count: diffHours } });
		if (diffDays < 7) return $_('chat.sidebar.time.days', { values: { count: diffDays } });

		return date.toLocaleDateString();
	}

	// Toggle sidebar on mobile
	function toggleSidebar() {
		chatStore.toggleSidebar();
	}
</script>

<!-- Sidebar overlay for mobile -->
{#if $sidebarOpen}
	<div
		class="fixed inset-0 z-40 bg-black/50 lg:hidden"
		onclick={toggleSidebar}
		onkeydown={(e) => e.key === 'Escape' && toggleSidebar()}
		role="button"
		tabindex="0"
		aria-label={$_('chat.sidebar.close')}
	></div>
{/if}

<!-- Modern sidebar -->
<aside
	class="bg-background-secondary text-text-primary fixed top-16 left-0 z-50 flex h-[calc(100vh-4rem)] w-80 flex-col transition-transform lg:relative lg:top-0 lg:z-auto lg:h-full lg:translate-x-0 {$sidebarOpen
		? 'translate-x-0'
		: '-translate-x-full'}"
	aria-label={$_('chat.sidebar.label')}
>
	<!-- Header with new chat button -->
	<div class="flex items-center justify-between p-4">
		<div class="flex items-center gap-3">
			<h2 class="text-lg font-semibold">{$_('chat.sidebar.title')}</h2>
		</div>

		<!-- Mobile close button -->
		<Button
			variant="ghost"
			size="sm"
			class="text-text-muted hover:bg-background-tertiary hover:text-text-primary h-8 w-8 p-0 lg:hidden"
			onclick={toggleSidebar}
			aria-label={$_('chat.sidebar.close')}
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

	<!-- New conversation button -->
	<div class="px-4 pb-4">
		<Button
			variant="outline"
			class="border-border text-text-primary hover:bg-background-tertiary w-full justify-start bg-transparent"
			onclick={startNewConversation}
		>
			<svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M12 6v6m0 0v6m0-6h6m-6 0H6"
				/>
			</svg>
			{$_('chat.sidebar.newChat')}
		</Button>
	</div>

	<!-- Conversations list -->
	<div class="flex-1 overflow-y-auto px-2">
		{#if $isLoading}
			<!-- Loading state -->
			<div class="flex items-center justify-center p-8">
				<div class="text-center">
					<div
						class="border-border border-t-text-primary mx-auto mb-4 h-6 w-6 animate-spin rounded-full border-2"
					></div>
					<p class="text-text-muted text-sm">{$_('chat.sidebar.loading')}</p>
				</div>
			</div>
		{:else if $conversations.length === 0}
			<!-- Empty state -->
			<div class="flex items-center justify-center p-8">
				<div class="text-center">
					<svg
						class="text-text-muted mx-auto mb-4 h-12 w-12"
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
					<p class="text-text-secondary mb-2 text-sm font-medium">
						{$_('chat.sidebar.empty.title')}
					</p>
					<p class="text-text-muted text-xs">{$_('chat.sidebar.empty.description')}</p>
				</div>
			</div>
		{:else}
			<!-- Conversations list -->
			<div class="space-y-1">
				{#each $conversations as conversation (conversation.id)}
					<div class="group relative">
						<button
							class="hover:bg-background-tertiary flex w-full items-center gap-3 rounded-lg p-3 text-left transition-colors {$currentConversation?.id ===
							conversation.id
								? 'bg-background-tertiary'
								: ''}"
							onclick={() => loadConversation(conversation.id)}
							aria-current={$currentConversation?.id === conversation.id ? 'page' : undefined}
						>
							<!-- Chat icon -->
							<div class="flex-shrink-0">
								<svg
									class="text-text-muted h-4 w-4"
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

							<!-- Conversation content -->
							<div class="min-w-0 flex-1">
								<div class="text-text-primary truncate text-sm font-medium">
									{getConversationTitle(conversation)}
								</div>

								{#if conversation.last_message}
									<div class="text-text-secondary mt-1 truncate text-xs">
										{conversation.last_message.content}
									</div>
								{/if}

								<div class="text-text-muted mt-1 text-xs">
									{getRelativeTime(conversation.updated_at)}
								</div>
							</div>

							<!-- Action buttons (visible on hover) -->
							<div class="flex-shrink-0 opacity-0 transition-opacity group-hover:opacity-100">
								<div class="flex gap-1">
									<Button
										variant="ghost"
										size="sm"
										class="text-text-muted hover:bg-background-tertiary hover:text-text-primary h-8 w-8 p-0 transition-colors"
										onclick={(e) => archiveConversation(conversation.id, e)}
										aria-label={$_('chat.sidebar.archive')}
									>
										<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M5 8l4 4 4-4"
											/>
										</svg>
									</Button>

									<Button
										variant="ghost"
										size="sm"
										class="text-text-muted hover:bg-status-error h-8 w-8 p-0 transition-colors hover:text-white"
										onclick={(e) => deleteConversation(conversation.id, e)}
										aria-label={$_('chat.sidebar.delete')}
									>
										<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
											/>
										</svg>
									</Button>
								</div>
							</div>
						</button>
					</div>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Footer -->
	<div class="border-border p-4">
		<div class="text-text-muted text-xs">
			{$_('chat.sidebar.footer', { values: { count: $conversations.length } })}
		</div>
	</div>
</aside>
