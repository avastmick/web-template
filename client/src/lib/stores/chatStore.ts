// Chat store for managing chat state and AI interactions

import { writable, derived, get } from 'svelte/store';
import type {
	ChatState,
	Conversation,
	ChatMessage,
	StreamEvent,
	ChatRequest,
	ChatResponse
} from '$lib/types/chat.js';
import * as aiClient from '$lib/services/aiClient.js';

// Initial state
const initialState: ChatState = {
	currentConversation: null,
	conversations: [],
	isLoading: false,
	isStreaming: false,
	error: null,
	inputText: '',
	uploadedFiles: [],
	uploadedFileResponses: [],
	isUploading: false,
	sidebarOpen: true
};

// Create the main chat store
function createChatStore() {
	const { subscribe, set, update } = writable<ChatState>(initialState);

	// Store for abort function to cancel streaming
	let abortStreaming: (() => void) | null = null;

	return {
		subscribe,

		// Actions
		async loadConversations() {
			update((state) => ({ ...state, isLoading: true, error: null }));

			try {
				const response = await aiClient.getConversations();
				update((state) => ({
					...state,
					conversations: response.conversations,
					isLoading: false
				}));
			} catch (error) {
				update((state) => ({
					...state,
					error: error instanceof Error ? error.message : 'Failed to load conversations',
					isLoading: false
				}));
			}
		},

		async loadConversation(conversationId: string) {
			update((state) => ({ ...state, isLoading: true, error: null }));

			try {
				const response = await aiClient.getConversation(conversationId);
				update((state) => ({
					...state,
					currentConversation: {
						...response.conversation,
						messages: response.messages
					},
					isLoading: false
				}));
			} catch (error) {
				update((state) => ({
					...state,
					error: error instanceof Error ? error.message : 'Failed to load conversation',
					isLoading: false
				}));
			}
		},

		startNewConversation() {
			update((state) => ({
				...state,
				currentConversation: null,
				error: null
			}));
		},

		setInputText(text: string) {
			update((state) => ({ ...state, inputText: text }));
		},

		async sendMessage(content: string, useStreaming = false) {
			const state = get({ subscribe });
			if (!content.trim() || state.isStreaming) return;

			// Build message content with file context
			let fullContent = content.trim();

			if (state.uploadedFileResponses.length > 0) {
				const fileContext = state.uploadedFileResponses
					.map((file) => {
						// For text files, include the content directly
						// For binary files (base64), include a reference
						const isBase64 = file.content.match(/^[A-Za-z0-9+/]+=*$/);
						if (isBase64) {
							return `[Attached file: ${file.name} (${file.content_type}, ${file.size} bytes) - Binary content]`;
						} else {
							return `--- File: ${file.name} ---\n${file.content}\n--- End of ${file.name} ---`;
						}
					})
					.join('\n\n');

				fullContent = `${fileContext}\n\nUser message: ${content.trim()}`;
			}

			// Create user message - show original message in UI, but send full content to API
			const userMessage: ChatMessage = {
				id: crypto.randomUUID(),
				role: 'user',
				content: content.trim(), // Show only the user's message in UI
				timestamp: new Date().toISOString(),
				metadata:
					state.uploadedFileResponses.length > 0
						? {
								attachedFiles: state.uploadedFiles.map((f) => f.name)
							}
						: undefined
			};

			// Add user message to current conversation
			update((s) => {
				if (s.currentConversation) {
					// Add to existing conversation
					return {
						...s,
						currentConversation: {
							...s.currentConversation,
							messages: [...s.currentConversation.messages, userMessage]
						},
						inputText: '',
						error: null,
						isStreaming: useStreaming
					};
				} else {
					// Create temporary conversation to show the user message
					return {
						...s,
						currentConversation: {
							id: 'temp-' + Date.now(), // Temporary ID
							messages: [userMessage],
							created_at: new Date().toISOString(),
							updated_at: new Date().toISOString()
						} as Conversation,
						inputText: '',
						error: null,
						isStreaming: useStreaming
					};
				}
			});

			// Prepare chat request - only send the message, not the full history for new conversations
			const currentState = get({ subscribe });
			const messages = currentState.currentConversation?.id.startsWith('temp-')
				? [{ role: userMessage.role, content: fullContent }] // Use full content with files
				: currentState.currentConversation?.messages.map((msg, index) => ({
						role: msg.role,
						// Use full content for the last message (current one), original for others
						content:
							index === currentState.currentConversation!.messages.length - 1
								? fullContent
								: msg.content
					})) || [];

			const request: ChatRequest = {
				messages
			};

			try {
				if (useStreaming) {
					await this.sendStreamingMessage(request);
				} else {
					const response = await aiClient.sendChatMessage(request);
					this.handleChatResponse(response);
				}
			} catch (error) {
				update((s) => ({
					...s,
					error: error instanceof Error ? error.message : 'Failed to send message',
					isStreaming: false
				}));
			}
		},

		async sendStreamingMessage(request: ChatRequest) {
			// Create placeholder for AI response
			const assistantMessage: ChatMessage = {
				id: crypto.randomUUID(),
				role: 'assistant',
				content: '',
				timestamp: new Date().toISOString()
			};

			// Add placeholder message
			update((s) => ({
				...s,
				currentConversation: s.currentConversation
					? {
							...s.currentConversation,
							messages: [...s.currentConversation.messages, assistantMessage]
						}
					: null
			}));

			// Start streaming
			abortStreaming = aiClient.sendChatMessageStream(
				request,
				(event: StreamEvent) => {
					update((s) => {
						if (!s.currentConversation) return s;

						const messages = [...s.currentConversation.messages];
						const lastMessage = messages[messages.length - 1];

						if (lastMessage && lastMessage.role === 'assistant') {
							if (event.type === 'delta' && event.content) {
								lastMessage.content += event.content;
							}
						}

						return {
							...s,
							currentConversation: {
								...s.currentConversation,
								messages
							}
						};
					});
				},
				(error: Error) => {
					update((s) => ({
						...s,
						error: error.message,
						isStreaming: false
					}));
					abortStreaming = null;
				},
				() => {
					update((s) => ({
						...s,
						isStreaming: false
					}));
					abortStreaming = null;

					// Reload conversations to get updated list
					this.loadConversations();
				}
			);
		},

		handleChatResponse(response: ChatResponse) {
			update((s) => {
				if (!s.currentConversation) {
					// This shouldn't happen, but handle it gracefully
					return {
						...s,
						currentConversation: {
							id: response.conversation_id,
							messages: [response.message],
							created_at: new Date().toISOString(),
							updated_at: new Date().toISOString()
						},
						isStreaming: false
					};
				}

				// Replace temporary ID with real conversation ID from server
				const isNewConversation = s.currentConversation.id.startsWith('temp-');
				const updatedConversation: Conversation = {
					...s.currentConversation,
					id: response.conversation_id,
					messages: isNewConversation
						? [...s.currentConversation.messages, response.message] // Keep user message and add response
						: [...s.currentConversation.messages, response.message]
				};

				return {
					...s,
					currentConversation: updatedConversation,
					isStreaming: false
				};
			});

			// Reload conversations to get updated list
			this.loadConversations();
		},

		stopStreaming() {
			if (abortStreaming) {
				abortStreaming();
				abortStreaming = null;
			}
			update((s) => ({ ...s, isStreaming: false }));
		},

		async archiveConversation(conversationId: string) {
			try {
				await aiClient.archiveConversation(conversationId);

				// Remove from conversations list and clear current if it's the archived one
				update((s) => ({
					...s,
					conversations: s.conversations.filter((conv) => conv.id !== conversationId),
					currentConversation:
						s.currentConversation?.id === conversationId ? null : s.currentConversation
				}));
			} catch (error) {
				update((s) => ({
					...s,
					error: error instanceof Error ? error.message : 'Failed to archive conversation'
				}));
			}
		},

		async deleteConversation(conversationId: string) {
			try {
				await aiClient.deleteConversation(conversationId);

				// Remove from conversations list and clear current if it's the deleted one
				update((s) => ({
					...s,
					conversations: s.conversations.filter((conv) => conv.id !== conversationId),
					currentConversation:
						s.currentConversation?.id === conversationId ? null : s.currentConversation
				}));
			} catch (error) {
				update((s) => ({
					...s,
					error: error instanceof Error ? error.message : 'Failed to delete conversation'
				}));
			}
		},

		async uploadFiles(files: File[]) {
			update((s) => ({ ...s, isUploading: true, error: null }));

			try {
				const response = await aiClient.uploadFiles(files);
				update((s) => ({
					...s,
					uploadedFiles: [...s.uploadedFiles, ...files],
					uploadedFileResponses: [...s.uploadedFileResponses, ...response],
					isUploading: false
				}));
				return response;
			} catch (error) {
				update((s) => ({
					...s,
					error: error instanceof Error ? error.message : 'Failed to upload files',
					isUploading: false
				}));
				throw error;
			}
		},

		removeUploadedFile(index: number) {
			update((s) => ({
				...s,
				uploadedFiles: s.uploadedFiles.filter((_, i) => i !== index),
				uploadedFileResponses: s.uploadedFileResponses.filter((_, i) => i !== index)
			}));
		},

		clearUploadedFiles() {
			update((s) => ({ ...s, uploadedFiles: [], uploadedFileResponses: [] }));
		},

		toggleSidebar() {
			update((s) => ({ ...s, sidebarOpen: !s.sidebarOpen }));
		},

		setSidebarOpen(open: boolean) {
			update((s) => ({ ...s, sidebarOpen: open }));
		},

		clearError() {
			update((s) => ({ ...s, error: null }));
		},

		setError(error: string) {
			update((s) => ({ ...s, error }));
		},

		reset() {
			// Stop any ongoing streaming
			if (abortStreaming) {
				abortStreaming();
				abortStreaming = null;
			}
			set(initialState);
		}
	};
}

// Create and export the chat store
export const chatStore = createChatStore();

// Derived stores for convenience
export const currentConversation = derived(chatStore, ($state) => $state.currentConversation);
export const conversations = derived(chatStore, ($state) => $state.conversations);
export const isLoading = derived(chatStore, ($state) => $state.isLoading);
export const isStreaming = derived(chatStore, ($state) => $state.isStreaming);
export const error = derived(chatStore, ($state) => $state.error);
export const inputText = derived(chatStore, ($state) => $state.inputText);
export const uploadedFiles = derived(chatStore, ($state) => $state.uploadedFiles);
export const uploadedFileResponses = derived(chatStore, ($state) => $state.uploadedFileResponses);
export const isUploading = derived(chatStore, ($state) => $state.isUploading);
export const sidebarOpen = derived(chatStore, ($state) => $state.sidebarOpen);
