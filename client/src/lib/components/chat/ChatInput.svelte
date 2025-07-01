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
					class="border-border-default bg-surface-secondary flex items-center gap-2 rounded-lg border px-3 py-2 text-sm"
				>
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
							d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
						/>
					</svg>
					<span class="text-text-primary truncate">{file.name}</span>
					<Button
						variant="ghost"
						size="sm"
						class="text-text-muted hover:text-error h-4 w-4 p-0 transition-colors"
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
		class="border-border-default bg-surface-primary relative flex items-end gap-2 rounded-2xl border p-4 shadow-sm transition-colors {isDragOver
			? 'border-primary bg-primary/10'
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
			class="text-text-secondary hover:text-primary hover:bg-background-secondary h-10 w-10 flex-shrink-0 p-0 transition-colors"
			onclick={() => fileInput?.click()}
			disabled={$isUploading}
			aria-label={$_('chat.input.uploadFile')}
		>
			{#if $isUploading}
				<svg class="h-8 w-8 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
					/>
				</svg>
			{:else}
				<svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
			class="text-text-primary placeholder-text-muted focus-visible:ring-focus flex-1 resize-none border-none bg-transparent focus:outline-none focus-visible:ring-2"
			rows="1"
			disabled={$isStreaming}
			aria-label={$_('chat.input.message')}
		></textarea>

		<!-- Send/Stop button -->
		{#if $isStreaming}
			<Button
				variant="ghost"
				size="sm"
				class="text-text-secondary hover:text-error hover:bg-background-secondary h-10 w-10 flex-shrink-0 p-0 transition-colors"
				onclick={handleStop}
				aria-label={$_('chat.input.stop')}
			>
				<svg class="h-8 w-8" fill="currentColor" viewBox="0 0 24 24">
					<rect width="8" height="8" x="8" y="8" rx="1" />
				</svg>
			</Button>
		{:else}
			<Button
				variant="ghost"
				size="sm"
				class="h-10 w-10 flex-shrink-0 p-0 transition-colors {$inputText.trim()
					? 'bg-primary text-text-inverse hover:bg-primary-hover'
					: 'text-text-muted hover:bg-background-secondary hover:text-text-secondary'}"
				onclick={handleSubmit}
				disabled={!$inputText.trim() || $isUploading}
				aria-label={$_('chat.input.send')}
			>
				<svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
				class="border-primary bg-primary/20 absolute inset-0 flex items-center justify-center rounded-2xl border-2 border-dashed"
			>
				<div class="text-center">
					<svg
						class="text-primary mx-auto h-8 w-8"
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
					<p class="text-primary mt-2 text-sm font-medium">
						{$_('chat.input.dropFiles')}
					</p>
				</div>
			</div>
		{/if}
	</div>

	<!-- Input hints -->
	<div class="text-text-muted mt-2 flex items-center justify-between text-xs">
		<div class="flex items-center gap-4">
			<span>{$_('chat.input.hint.enter')}</span>
			<span>{$_('chat.input.hint.shiftEnter')}</span>
		</div>
		{#if $uploadedFiles.length > 0}
			<span>{$_('chat.input.filesUploaded', { values: { count: $uploadedFiles.length } })}</span>
		{/if}
	</div>
</div>
