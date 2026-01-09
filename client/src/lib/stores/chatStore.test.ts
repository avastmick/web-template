import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import type {
	ChatResponse,
	ConversationListResponse,
	Conversation,
	ChatMessage
} from '$lib/types/chat';

// Mock aiClient
const mockAiClient = {
	getConversations: vi.fn(),
	getConversation: vi.fn(),
	sendChatMessage: vi.fn(),
	sendChatMessageStream: vi.fn(),
	archiveConversation: vi.fn(),
	deleteConversation: vi.fn(),
	uploadFiles: vi.fn()
};

vi.mock('$lib/services/aiClient.js', () => mockAiClient);

// Import after mocks are set up
const {
	chatStore,
	currentConversation,
	conversations,
	isLoading,
	isStreaming,
	error,
	inputText,
	uploadedFiles,
	sidebarOpen
} = await import('./chatStore');

describe('chatStore', () => {
	const mockConversation: Conversation = {
		id: 'conv-123',
		title: 'Test Conversation',
		messages: [],
		created_at: '2024-01-01T00:00:00Z',
		updated_at: '2024-01-01T00:00:00Z'
	};

	const mockMessage: ChatMessage = {
		id: 'msg-123',
		role: 'assistant',
		content: 'Hello, how can I help?',
		timestamp: '2024-01-01T00:00:00Z'
	};

	const mockConversationList: ConversationListResponse = {
		conversations: [
			{
				id: 'conv-123',
				title: 'Test Conversation',
				created_at: '2024-01-01T00:00:00Z',
				updated_at: '2024-01-01T00:00:00Z',
				message_count: 2
			}
		],
		total: 1
	};

	beforeEach(() => {
		vi.clearAllMocks();
		chatStore.reset();
	});

	describe('initial state', () => {
		it('should have correct initial state', () => {
			expect(get(currentConversation)).toBeNull();
			expect(get(conversations)).toEqual([]);
			expect(get(isLoading)).toBe(false);
			expect(get(isStreaming)).toBe(false);
			expect(get(error)).toBeNull();
			expect(get(inputText)).toBe('');
			expect(get(uploadedFiles)).toEqual([]);
			expect(get(sidebarOpen)).toBe(true);
		});
	});

	describe('loadConversations', () => {
		it('should load conversations successfully', async () => {
			mockAiClient.getConversations.mockResolvedValue(mockConversationList);

			await chatStore.loadConversations();

			expect(get(conversations)).toEqual(mockConversationList.conversations);
			expect(get(isLoading)).toBe(false);
			expect(get(error)).toBeNull();
		});

		it('should set loading state while fetching', async () => {
			let resolvePromise: (value: ConversationListResponse) => void;
			const promise = new Promise<ConversationListResponse>((resolve) => {
				resolvePromise = resolve;
			});
			mockAiClient.getConversations.mockReturnValue(promise);

			const loadPromise = chatStore.loadConversations();

			expect(get(isLoading)).toBe(true);

			resolvePromise!(mockConversationList);
			await loadPromise;

			expect(get(isLoading)).toBe(false);
		});

		it('should handle errors', async () => {
			mockAiClient.getConversations.mockRejectedValue(new Error('Network error'));

			await chatStore.loadConversations();

			expect(get(error)).toBe('Network error');
			expect(get(isLoading)).toBe(false);
		});
	});

	describe('loadConversation', () => {
		it('should load a specific conversation', async () => {
			const conversationWithMessages = {
				conversation: mockConversation,
				messages: [mockMessage]
			};
			mockAiClient.getConversation.mockResolvedValue(conversationWithMessages);

			await chatStore.loadConversation('conv-123');

			const current = get(currentConversation);
			expect(current?.id).toBe('conv-123');
			expect(current?.messages).toEqual([mockMessage]);
		});

		it('should handle load errors', async () => {
			mockAiClient.getConversation.mockRejectedValue(new Error('Not found'));

			await chatStore.loadConversation('invalid-id');

			expect(get(error)).toBe('Not found');
		});
	});

	describe('startNewConversation', () => {
		it('should clear current conversation', async () => {
			// Load a conversation first
			mockAiClient.getConversation.mockResolvedValue({
				conversation: mockConversation,
				messages: []
			});
			await chatStore.loadConversation('conv-123');

			chatStore.startNewConversation();

			expect(get(currentConversation)).toBeNull();
			expect(get(error)).toBeNull();
		});
	});

	describe('setInputText', () => {
		it('should update input text', () => {
			chatStore.setInputText('Hello world');
			expect(get(inputText)).toBe('Hello world');
		});
	});

	describe('sendMessage', () => {
		it('should send a message and receive response', async () => {
			const response: ChatResponse = {
				conversation_id: 'conv-new',
				message: mockMessage
			};
			mockAiClient.sendChatMessage.mockResolvedValue(response);
			mockAiClient.getConversations.mockResolvedValue(mockConversationList);

			await chatStore.sendMessage('Hello', false);

			// Should have called sendChatMessage
			expect(mockAiClient.sendChatMessage).toHaveBeenCalled();

			// Conversation should be updated
			const current = get(currentConversation);
			expect(current?.messages.length).toBeGreaterThan(0);
		});

		it('should not send empty messages', async () => {
			await chatStore.sendMessage('', false);
			expect(mockAiClient.sendChatMessage).not.toHaveBeenCalled();

			await chatStore.sendMessage('   ', false);
			expect(mockAiClient.sendChatMessage).not.toHaveBeenCalled();
		});

		it('should clear input after sending', async () => {
			chatStore.setInputText('Test message');

			const response: ChatResponse = {
				conversation_id: 'conv-new',
				message: mockMessage
			};
			mockAiClient.sendChatMessage.mockResolvedValue(response);
			mockAiClient.getConversations.mockResolvedValue(mockConversationList);

			await chatStore.sendMessage('Test message', false);

			expect(get(inputText)).toBe('');
		});

		it('should handle send errors', async () => {
			mockAiClient.sendChatMessage.mockRejectedValue(new Error('Send failed'));

			await chatStore.sendMessage('Hello', false);

			expect(get(error)).toBe('Send failed');
		});
	});

	describe('streaming', () => {
		it('should set streaming state during stream', async () => {
			// Mock streaming to return an abort function
			mockAiClient.sendChatMessageStream.mockReturnValue(() => {});
			mockAiClient.getConversations.mockResolvedValue(mockConversationList);

			// Send with streaming enabled
			await chatStore.sendMessage('Hello', true);

			// The store should track streaming state
			// Note: actual streaming completion would call onComplete callback
		});

		it('should stop streaming when requested', () => {
			const mockAbort = vi.fn();
			mockAiClient.sendChatMessageStream.mockReturnValue(mockAbort);

			chatStore.stopStreaming();

			expect(get(isStreaming)).toBe(false);
		});
	});

	describe('archiveConversation', () => {
		it('should archive and remove conversation from list', async () => {
			// Set up initial conversations
			mockAiClient.getConversations.mockResolvedValue(mockConversationList);
			await chatStore.loadConversations();

			mockAiClient.archiveConversation.mockResolvedValue(undefined);

			await chatStore.archiveConversation('conv-123');

			expect(mockAiClient.archiveConversation).toHaveBeenCalledWith('conv-123');
			expect(get(conversations).find((c) => c.id === 'conv-123')).toBeUndefined();
		});

		it('should clear current conversation if archived', async () => {
			mockAiClient.getConversation.mockResolvedValue({
				conversation: mockConversation,
				messages: []
			});
			await chatStore.loadConversation('conv-123');

			mockAiClient.archiveConversation.mockResolvedValue(undefined);
			await chatStore.archiveConversation('conv-123');

			expect(get(currentConversation)).toBeNull();
		});
	});

	describe('deleteConversation', () => {
		it('should delete and remove conversation from list', async () => {
			mockAiClient.getConversations.mockResolvedValue(mockConversationList);
			await chatStore.loadConversations();

			mockAiClient.deleteConversation.mockResolvedValue(undefined);

			await chatStore.deleteConversation('conv-123');

			expect(mockAiClient.deleteConversation).toHaveBeenCalledWith('conv-123');
			expect(get(conversations).find((c) => c.id === 'conv-123')).toBeUndefined();
		});

		it('should handle delete errors', async () => {
			mockAiClient.deleteConversation.mockRejectedValue(new Error('Delete failed'));

			await chatStore.deleteConversation('conv-123');

			expect(get(error)).toBe('Delete failed');
		});
	});

	describe('file uploads', () => {
		it('should upload files successfully', async () => {
			const mockFiles = [new File(['content'], 'test.txt', { type: 'text/plain' })];
			const mockResponse = [{ name: 'test.txt', content: 'content', size: 7 }];

			mockAiClient.uploadFiles.mockResolvedValue(mockResponse);

			await chatStore.uploadFiles(mockFiles);

			expect(get(uploadedFiles)).toEqual(mockFiles);
		});

		it('should handle upload errors', async () => {
			mockAiClient.uploadFiles.mockRejectedValue(new Error('Upload failed'));

			await expect(chatStore.uploadFiles([new File([''], 'test.txt')])).rejects.toThrow(
				'Upload failed'
			);
			expect(get(error)).toBe('Upload failed');
		});

		it('should remove uploaded files', async () => {
			const mockFiles = [
				new File(['a'], 'a.txt'),
				new File(['b'], 'b.txt'),
				new File(['c'], 'c.txt')
			];
			mockAiClient.uploadFiles.mockResolvedValue(mockFiles.map((f) => ({ name: f.name })));
			await chatStore.uploadFiles(mockFiles);

			chatStore.removeUploadedFile(1);

			expect(get(uploadedFiles)).toHaveLength(2);
		});

		it('should clear all uploaded files', async () => {
			const mockFiles = [new File(['a'], 'a.txt')];
			mockAiClient.uploadFiles.mockResolvedValue([{ name: 'a.txt' }]);
			await chatStore.uploadFiles(mockFiles);

			chatStore.clearUploadedFiles();

			expect(get(uploadedFiles)).toEqual([]);
		});
	});

	describe('sidebar', () => {
		it('should toggle sidebar', () => {
			expect(get(sidebarOpen)).toBe(true);

			chatStore.toggleSidebar();
			expect(get(sidebarOpen)).toBe(false);

			chatStore.toggleSidebar();
			expect(get(sidebarOpen)).toBe(true);
		});

		it('should set sidebar open state', () => {
			chatStore.setSidebarOpen(false);
			expect(get(sidebarOpen)).toBe(false);

			chatStore.setSidebarOpen(true);
			expect(get(sidebarOpen)).toBe(true);
		});
	});

	describe('error handling', () => {
		it('should set error', () => {
			chatStore.setError('Test error');
			expect(get(error)).toBe('Test error');
		});

		it('should clear error', () => {
			chatStore.setError('Test error');
			chatStore.clearError();
			expect(get(error)).toBeNull();
		});
	});

	describe('reset', () => {
		it('should reset to initial state', async () => {
			mockAiClient.getConversations.mockResolvedValue(mockConversationList);
			await chatStore.loadConversations();
			chatStore.setInputText('test');
			chatStore.setError('error');

			chatStore.reset();

			expect(get(currentConversation)).toBeNull();
			expect(get(conversations)).toEqual([]);
			expect(get(isLoading)).toBe(false);
			expect(get(inputText)).toBe('');
			expect(get(error)).toBeNull();
		});
	});
});
