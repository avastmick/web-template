<!-- Modern chat input component inspired by ChatGPT -->

<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { _ } from 'svelte-i18n';
	import { Button } from '$lib/components/ui/index.js';
	import {
		chatStore,
		inputText,
		uploadedFiles,
		isUploading,
		isStreaming
	} from '$lib/stores/chatStore.js';

	const dispatch = createEventDispatcher<{
		send: { message: string; files: File[] };
		fileUpload: { files: File[] };
	}>();

	let inputElement: HTMLTextAreaElement;
	let fileInput: HTMLInputElement;
	let isDragOver = false;

	// Auto-resize textarea
	function autoResize() {
		if (inputElement) {
			inputElement.style.height = 'auto';
			inputElement.style.height = Math.min(inputElement.scrollHeight, 120) + 'px';
		}
	}

	// Handle form submission
	function handleSubmit() {
		const message = $inputText.trim();
		if (!message || $isStreaming) return;

		dispatch('send', {
			message,
			files: $uploadedFiles
		});

		chatStore.sendMessage(message);
		chatStore.clearUploadedFiles();
	}

	// Handle keyboard shortcuts
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			handleSubmit();
		}
	}

	// Handle file input change
	function handleFileChange(event: Event) {
		const target = event.target as HTMLInputElement;
		const files = Array.from(target.files || []);
		if (files.length > 0) {
			handleFileUpload(files);
		}
		target.value = ''; // Reset input
	}

	// Handle file upload
	async function handleFileUpload(files: File[]) {
		try {
			await chatStore.uploadFiles(files);
			dispatch('fileUpload', { files });
		} catch (error) {
			console.error('File upload failed:', error);
			chatStore.setError(
				error instanceof Error ? error.message : 'Failed to upload files. Please try again.'
			);
		}
	}

	// Drag and drop handlers
	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		isDragOver = true;
	}

	function handleDragLeave(event: DragEvent) {
		event.preventDefault();
		isDragOver = false;
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		isDragOver = false;

		const files = Array.from(event.dataTransfer?.files || []);
		if (files.length > 0) {
			handleFileUpload(files);
		}
	}

	// Remove uploaded file
	function removeFile(index: number) {
		chatStore.removeUploadedFile(index);
	}

	// Stop streaming
	function handleStop() {
		chatStore.stopStreaming();
	}

	// Focus input when component mounts
	import { onMount } from 'svelte';
	onMount(() => {
		inputElement?.focus();
	});
</script>

<!-- Modern ChatGPT-style input -->
<div class="mx-auto w-full max-w-3xl p-4">
	<!-- Uploaded files display -->
	{#if $uploadedFiles.length > 0}
		<div class="mb-3 flex flex-wrap gap-2">
			{#each $uploadedFiles as file, index (file.name)}
				<div
					class="flex items-center gap-2 rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm dark:border-gray-700 dark:bg-gray-800"
				>
					<svg class="h-4 w-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
						/>
					</svg>
					<span class="truncate text-gray-700 dark:text-gray-300">{file.name}</span>
					<Button
						variant="ghost"
						size="sm"
						class="h-4 w-4 p-0 text-gray-500 hover:text-red-500"
						onclick={() => removeFile(index)}
						aria-label={$_('chat.input.removeFile', { values: { filename: file.name } })}
					>
						<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M6 18L18 6M6 6l12 12"
							/>
						</svg>
					</Button>
				</div>
			{/each}
		</div>
	{/if}

	<!-- Main input container -->
	<div
		class="relative flex items-end gap-2 rounded-2xl border border-gray-200 bg-white p-4 shadow-sm transition-colors dark:border-gray-700 dark:bg-gray-800 {isDragOver
			? 'border-blue-500 bg-blue-50 dark:border-blue-400 dark:bg-blue-900/20'
			: ''}"
		ondragover={handleDragOver}
		ondragleave={handleDragLeave}
		ondrop={handleDrop}
		role="region"
		aria-label={$_('chat.input.region')}
	>
		<!-- File upload input -->
		<input
			bind:this={fileInput}
			type="file"
			multiple
			accept=".txt,.md,.csv,.json,.xml,.yaml,.yml,.log,.conf,.ini,.toml,.js,.ts,.jsx,.tsx,.py,.java,.c,.cpp,.h,.hpp,.rs,.go,.rb,.php,.css,.html,.vue,.svelte,.pdf,.doc,.docx"
			class="hidden"
			onchange={handleFileChange}
		/>

		<!-- File upload button -->
		<Button
			variant="ghost"
			size="sm"
			class="h-8 w-8 flex-shrink-0 p-0 text-gray-500 hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-300"
			onclick={() => fileInput?.click()}
			disabled={$isUploading}
			aria-label={$_('chat.input.uploadFile')}
		>
			{#if $isUploading}
				<svg class="h-4 w-4 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
					/>
				</svg>
			{:else}
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"
					/>
				</svg>
			{/if}
		</Button>

		<!-- Text input -->
		<textarea
			bind:this={inputElement}
			value={$inputText}
			oninput={(e) => {
				chatStore.setInputText(e.currentTarget.value);
				autoResize();
			}}
			onkeydown={handleKeydown}
			placeholder={$_('chat.input.placeholder')}
			class="flex-1 resize-none border-none bg-transparent text-gray-900 placeholder-gray-500 focus:outline-none focus-visible:ring-amber-300 dark:text-white dark:placeholder-gray-400"
			rows="1"
			disabled={$isStreaming}
			aria-label={$_('chat.input.message')}
		></textarea>

		<!-- Send/Stop button -->
		{#if $isStreaming}
			<Button
				variant="ghost"
				size="sm"
				class="h-8 w-8 flex-shrink-0 p-0 text-gray-500 hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-300"
				onclick={handleStop}
				aria-label={$_('chat.input.stop')}
			>
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<rect width="6" height="6" x="9" y="9" rx="1" />
				</svg>
			</Button>
		{:else}
			<Button
				variant="ghost"
				size="sm"
				class="h-8 w-8 flex-shrink-0 p-0 {$inputText.trim()
					? 'bg-green-600 text-white hover:bg-green-700'
					: 'text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:text-gray-500 dark:hover:bg-gray-700 dark:hover:text-gray-400'}"
				onclick={handleSubmit}
				disabled={!$inputText.trim() || $isUploading}
				aria-label={$_('chat.input.send')}
			>
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
					/>
				</svg>
			</Button>
		{/if}

		<!-- Drag overlay -->
		{#if isDragOver}
			<div
				class="absolute inset-0 flex items-center justify-center rounded-2xl border-2 border-dashed border-blue-500 bg-blue-50/50 dark:border-blue-400 dark:bg-blue-900/20"
			>
				<div class="text-center">
					<svg
						class="mx-auto h-8 w-8 text-blue-600 dark:text-blue-400"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
						/>
					</svg>
					<p class="mt-2 text-sm font-medium text-blue-600 dark:text-blue-400">
						{$_('chat.input.dropFiles')}
					</p>
				</div>
			</div>
		{/if}
	</div>

	<!-- Input hints -->
	<div class="mt-2 flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
		<div class="flex items-center gap-4">
			<span>{$_('chat.input.hint.enter')}</span>
			<span>{$_('chat.input.hint.shiftEnter')}</span>
		</div>
		{#if $uploadedFiles.length > 0}
			<span>{$_('chat.input.filesUploaded', { values: { count: $uploadedFiles.length } })}</span>
		{/if}
	</div>
</div>
