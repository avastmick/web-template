<!-- Modern message bubble component inspired by ChatGPT -->

<script lang="ts">
	import type { ChatMessage } from '$lib/types/chat.js';
	import { _ } from 'svelte-i18n';
	import { Button } from '$lib/components/ui/index.js';

	export let message: ChatMessage;
	export let isStreaming = false;
	export let showActions = true;

	$: isUser = message.role === 'user';
	$: isAssistant = message.role === 'assistant';

	// Format timestamp
	$: formattedTime = new Date(message.timestamp).toLocaleTimeString([], {
		hour: '2-digit',
		minute: '2-digit'
	});

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
			<div class="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full bg-green-600">
				<svg class="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
				<div class="prose prose-sm max-w-none text-gray-900 dark:text-white">
					{#if isStreaming}
						<!-- Streaming content with cursor -->
						<span class="whitespace-pre-wrap">{message.content}</span>
						<span class="ml-1 inline-block h-4 w-0.5 animate-pulse bg-gray-600" aria-hidden="true"
						></span>
					{:else}
						<!-- Static content -->
						<div class="leading-relaxed whitespace-pre-wrap">
							{message.content}
						</div>
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
							class="h-6 w-6 p-0 text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-700 dark:hover:text-gray-300"
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
							class="h-6 w-6 p-0 text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-700 dark:hover:text-gray-300"
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
				<time
					class="mt-1 block text-xs text-gray-500 opacity-0 transition-opacity group-hover:opacity-100"
					datetime={message.timestamp}
				>
					{formattedTime}
				</time>
			</div>
		</div>
	{:else}
		<!-- User message -->
		<div class="flex max-w-[70%] flex-col items-end">
			<div class="rounded-2xl bg-blue-600 px-4 py-2 text-white">
				<div class="text-sm leading-relaxed whitespace-pre-wrap">
					{message.content}
				</div>
			</div>

			<!-- Timestamp -->
			<time
				class="mt-1 text-xs text-gray-500 opacity-0 transition-opacity group-hover:opacity-100"
				datetime={message.timestamp}
			>
				{formattedTime}
			</time>
		</div>
	{/if}
</div>
