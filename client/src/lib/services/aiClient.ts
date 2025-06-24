// AI client service for chat functionality

import type {
	ChatRequest,
	ChatResponse,
	Conversation,
	ConversationListResponse,
	StreamEvent,
	FileUploadResponse,
	UsageStats,
	AIInfo
} from '$lib/types/chat.js';

// Configuration - same pattern as apiAuth.ts
const SERVER_PORT = import.meta.env.SERVER_PORT || '8081';
const API_BASE_URL = `${window.location.protocol}//${window.location.hostname}:${SERVER_PORT}`;
const API_BASE = `${API_BASE_URL}/api/ai`;

/**
 * Get authentication headers with JWT token
 */
function getAuthHeaders(): HeadersInit {
	const token = localStorage.getItem('auth_token');
	return {
		'Content-Type': 'application/json',
		...(token && { Authorization: `Bearer ${token}` })
	};
}

/**
 * Handle API response errors
 */
async function handleResponse<T>(response: Response): Promise<T> {
	if (!response.ok) {
		const errorData = await response.json().catch(() => ({}));
		console.error('API Error:', errorData);
		throw new Error(
			errorData.message || errorData.error || `HTTP ${response.status}: ${response.statusText}`
		);
	}
	return response.json();
}

/**
 * Send a chat message and get AI response
 */
export async function sendChatMessage(request: ChatRequest): Promise<ChatResponse> {
	const response = await fetch(`${API_BASE}/chat`, {
		method: 'POST',
		headers: getAuthHeaders(),
		body: JSON.stringify(request)
	});

	return handleResponse<ChatResponse>(response);
}

/**
 * Send a chat message with streaming response
 */
export function sendChatMessageStream(
	request: ChatRequest,
	onEvent: (event: StreamEvent) => void,
	onError: (error: Error) => void,
	onComplete: () => void
): () => void {
	const controller = new AbortController();

	// Create URL with query parameters for streaming
	const url = new URL(`${API_BASE}/chat/stream`);

	fetch(url, {
		method: 'POST',
		headers: getAuthHeaders(),
		body: JSON.stringify({ ...request, stream: true }),
		signal: controller.signal
	})
		.then(async (response) => {
			if (!response.ok) {
				throw new Error(`HTTP ${response.status}: ${response.statusText}`);
			}

			const reader = response.body?.getReader();
			if (!reader) {
				throw new Error('Stream not available');
			}

			const decoder = new TextDecoder();
			let buffer = '';

			try {
				while (true) {
					const { done, value } = await reader.read();
					if (done) break;

					buffer += decoder.decode(value, { stream: true });
					const lines = buffer.split('\n');
					buffer = lines.pop() || '';

					for (const line of lines) {
						if (line.trim() === '') continue;
						if (line.startsWith('data: ')) {
							const data = line.slice(6);
							if (data === '[DONE]') {
								onComplete();
								return;
							}
							try {
								const event: StreamEvent = JSON.parse(data);
								onEvent(event);
							} catch {
								console.warn('Failed to parse SSE data:', data);
							}
						}
					}
				}
			} catch (error) {
				if (error instanceof Error && error.name !== 'AbortError') {
					onError(error);
				}
			} finally {
				reader.releaseLock();
			}
		})
		.catch((error) => {
			if (error.name !== 'AbortError') {
				onError(error);
			}
		});

	// Return abort function
	return () => controller.abort();
}

/**
 * Get list of conversations
 */
export async function getConversations(params?: {
	page?: number;
	limit?: number;
	archived?: boolean;
}): Promise<ConversationListResponse> {
	const url = new URL(`${API_BASE}/conversations`);

	if (params?.page) url.searchParams.set('page', params.page.toString());
	if (params?.limit) url.searchParams.set('limit', params.limit.toString());
	if (params?.archived !== undefined) url.searchParams.set('archived', params.archived.toString());

	const response = await fetch(url, {
		headers: getAuthHeaders()
	});

	return handleResponse<ConversationListResponse>(response);
}

/**
 * Get a specific conversation with messages
 */
export async function getConversation(conversationId: string): Promise<{
	conversation: Conversation;
	messages: Conversation['messages'];
}> {
	const response = await fetch(`${API_BASE}/conversations/${conversationId}`, {
		headers: getAuthHeaders()
	});

	return handleResponse<{ conversation: Conversation; messages: Conversation['messages'] }>(
		response
	);
}

/**
 * Archive a conversation
 */
export async function archiveConversation(conversationId: string): Promise<void> {
	const response = await fetch(`${API_BASE}/conversations/${conversationId}/archive`, {
		method: 'POST',
		headers: getAuthHeaders()
	});

	if (!response.ok) {
		const errorData = await response.json().catch(() => ({}));
		throw new Error(errorData.message || `Failed to archive conversation`);
	}
}

/**
 * Delete a conversation
 */
export async function deleteConversation(conversationId: string): Promise<void> {
	const response = await fetch(`${API_BASE}/conversations/${conversationId}`, {
		method: 'DELETE',
		headers: getAuthHeaders()
	});

	if (!response.ok) {
		const errorData = await response.json().catch(() => ({}));
		throw new Error(errorData.message || `Failed to delete conversation`);
	}
}

/**
 * Upload files for chat context
 */
export async function uploadFiles(files: File[]): Promise<FileUploadResponse[]> {
	const formData = new FormData();

	for (const file of files) {
		formData.append('files', file);
	}

	const token = localStorage.getItem('authToken');
	const headers: HeadersInit = {};
	if (token) {
		headers.Authorization = `Bearer ${token}`;
	}

	const response = await fetch(`${API_BASE}/upload`, {
		method: 'POST',
		headers,
		body: formData
	});

	return handleResponse<FileUploadResponse[]>(response);
}

/**
 * Send contextual chat with uploaded files
 */
export async function sendContextualChat(
	request: ChatRequest & {
		context?: string[];
		template?: string;
	}
): Promise<ChatResponse> {
	const response = await fetch(`${API_BASE}/chat/contextual`, {
		method: 'POST',
		headers: getAuthHeaders(),
		body: JSON.stringify(request)
	});

	return handleResponse<ChatResponse>(response);
}

/**
 * Get usage statistics
 */
export async function getUsageStats(): Promise<UsageStats> {
	const response = await fetch(`${API_BASE}/usage`, {
		headers: getAuthHeaders()
	});

	return handleResponse<UsageStats>(response);
}

/**
 * Get AI service information
 */
export async function getAIInfo(): Promise<AIInfo> {
	const response = await fetch(`${API_BASE}/info`, {
		headers: getAuthHeaders()
	});

	return handleResponse<AIInfo>(response);
}

/**
 * Check AI service health
 */
export async function checkAIHealth(): Promise<{ status: string; message?: string }> {
	const response = await fetch(`${API_BASE}/health`, {
		headers: getAuthHeaders()
	});

	return handleResponse<{ status: string; message?: string }>(response);
}
