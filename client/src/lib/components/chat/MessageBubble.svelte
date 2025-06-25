<!-- Modern message bubble component inspired by ChatGPT -->

<script lang="ts">
	import type { ChatMessage } from '$lib/types/chat.js';
	import { _ } from 'svelte-i18n';
	import { Button } from '$lib/components/ui/index.js';
	import MarkdownContent from './MarkdownContent.svelte';

	export let message: ChatMessage;
	export let isStreaming = false;
	export let showActions = true;

	$: isUser = message.role === 'user';
	$: isAssistant = message.role === 'assistant';

	// Format timestamp
	$: formattedTime = (() => {
		try {
			const date = new Date(message.timestamp);
			if (isNaN(date.getTime())) {
				return '';
			}
			return date.toLocaleTimeString([], {
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch {
			return '';
		}
	})();

	// Copy message content to clipboard
	async function copyMessage() {
		try {
			await navigator.clipboard.writeText(message.content);
			// TODO: Show toast notification
		} catch (err) {
			console.error('Failed to copy message:', err);
		}
	}

	// Regenerate AI response (placeholder for future implementation)
	function regenerateResponse() {
		// TODO: Implement regeneration
		console.log('Regenerate response for message:', message.id);
	}
</script>

<!-- Modern message bubble -->
<div class="group mb-6 flex w-full {isUser ? 'justify-end' : 'justify-start'}">
	{#if isAssistant}
		<!-- Assistant message -->
		<div class="flex w-full max-w-none gap-3">
			<!-- Avatar -->
			<div
				class="bg-color-action-primary flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full"
			>
				<svg
					class="text-text-inverse h-4 w-4"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
					/>
				</svg>
			</div>

			<!-- Message content -->
			<div class="min-w-0 flex-1">
				<div class="text-text-primary border-border-default bg-bg-secondary rounded-lg border p-4">
					{#if isStreaming}
						<!-- Streaming content with cursor (no markdown parsing while streaming) -->
						<div class="whitespace-pre-wrap">{message.content}</div>
						<span
							class="bg-text-secondary ml-1 inline-block h-4 w-0.5 animate-pulse"
							aria-hidden="true"
						></span>
					{:else}
						<!-- Static content with markdown rendering -->
						<MarkdownContent content={message.content} />
					{/if}
				</div>

				<!-- Message actions -->
				{#if showActions && !isStreaming}
					<div
						class="mt-2 flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100"
					>
						<Button
							variant="ghost"
							size="sm"
							class="text-text-secondary hover:bg-background-secondary hover:text-text-primary h-6 w-6 p-0"
							onclick={copyMessage}
							aria-label={$_('chat.message.copy')}
						>
							<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
								/>
							</svg>
						</Button>

						<Button
							variant="ghost"
							size="sm"
							class="text-text-secondary hover:bg-background-secondary hover:text-text-primary h-6 w-6 p-0"
							onclick={regenerateResponse}
							aria-label={$_('chat.message.regenerate')}
						>
							<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
								/>
							</svg>
						</Button>
					</div>
				{/if}

				<!-- Timestamp -->
				{#if formattedTime}
					<time
						class="text-text-muted mt-1 block text-xs opacity-0 transition-opacity group-hover:opacity-100"
						datetime={message.timestamp}
					>
						{formattedTime}
					</time>
				{/if}
			</div>
		</div>
	{:else}
		<!-- User message -->
		<div class="flex max-w-[70%] flex-col items-end">
			<!-- Attached files indicator -->
			{#if message.metadata?.attachedFiles && Array.isArray(message.metadata.attachedFiles) && message.metadata.attachedFiles.length > 0}
				<div class="mb-2 flex flex-wrap gap-1">
					{#each message.metadata.attachedFiles as fileName (fileName)}
						<div
							class="bg-background-secondary text-text-secondary flex items-center gap-1 rounded-md px-2 py-1 text-xs"
						>
							<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"
								/>
							</svg>
							<span>{fileName}</span>
						</div>
					{/each}
				</div>
			{/if}

			<div
				class="bg-color-action-primary text-text-inverse border-border-light rounded-2xl border px-4 py-2"
			>
				<div class="text-sm leading-relaxed whitespace-pre-wrap">
					{message.content}
				</div>
			</div>

			<!-- Timestamp -->
			{#if formattedTime}
				<time
					class="text-text-muted mt-1 text-xs opacity-0 transition-opacity group-hover:opacity-100"
					datetime={message.timestamp}
				>
					{formattedTime}
				</time>
			{/if}
		</div>
	{/if}
</div>
