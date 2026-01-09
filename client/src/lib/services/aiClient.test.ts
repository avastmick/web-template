import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock import.meta.env before imports
vi.stubGlobal('import.meta', {
	env: {
		SERVER_PORT: '8081'
	}
});

// Mock window.location
const mockLocation = {
	protocol: 'http:',
	hostname: 'localhost',
	href: ''
};
vi.stubGlobal('location', mockLocation);

// Now import the module under test
import {
	sendChatMessage,
	getConversations,
	getConversation,
	archiveConversation,
	deleteConversation,
	uploadFiles,
	sendContextualChat,
	getUsageStats,
	getAIInfo,
	checkAIHealth
} from './aiClient';

describe('aiClient', () => {
	const mockFetch = vi.fn();
	const API_BASE = 'http://localhost:8081/api/ai';

	beforeEach(() => {
		vi.stubGlobal('fetch', mockFetch);
		mockFetch.mockReset();
		vi.mocked(localStorage.getItem).mockReturnValue('test-token');
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	describe('sendChatMessage', () => {
		it('should send a chat message and return response', async () => {
			const mockResponse = {
				conversation_id: 'conv-123',
				message: {
					id: 'msg-1',
					role: 'assistant',
					content: 'Hello!',
					timestamp: '2026-01-10T12:00:00Z'
				},
				usage: {
					prompt_tokens: 10,
					completion_tokens: 5,
					total_tokens: 15
				}
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const request = {
				messages: [{ role: 'user' as const, content: 'Hello' }]
			};

			const result = await sendChatMessage(request);

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/chat`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				},
				body: JSON.stringify(request)
			});
			expect(result).toEqual(mockResponse);
		});

		it('should throw error on API failure', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 500,
				statusText: 'Internal Server Error',
				json: () => Promise.resolve({ message: 'Server error' })
			});

			const request = {
				messages: [{ role: 'user' as const, content: 'Hello' }]
			};

			await expect(sendChatMessage(request)).rejects.toThrow('Server error');
		});

		it('should handle JSON parse error in error response', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 500,
				statusText: 'Internal Server Error',
				json: () => Promise.reject(new Error('Invalid JSON'))
			});

			const request = {
				messages: [{ role: 'user' as const, content: 'Hello' }]
			};

			await expect(sendChatMessage(request)).rejects.toThrow('HTTP 500: Internal Server Error');
		});
	});

	describe('getConversations', () => {
		it('should fetch conversations list', async () => {
			const mockResponse = {
				conversations: [
					{
						id: 'conv-1',
						title: 'Test Conversation',
						created_at: '2026-01-10T12:00:00Z',
						updated_at: '2026-01-10T12:00:00Z',
						message_count: 5
					}
				],
				total: 1
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const result = await getConversations();

			expect(mockFetch).toHaveBeenCalledWith(
				expect.objectContaining({
					href: `${API_BASE}/conversations`
				}),
				{
					headers: {
						'Content-Type': 'application/json',
						Authorization: 'Bearer test-token'
					}
				}
			);
			expect(result).toEqual(mockResponse);
		});

		it('should include query parameters', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({ conversations: [], total: 0 })
			});

			await getConversations({ page: 2, limit: 10, archived: true });

			const calledUrl = mockFetch.mock.calls[0][0] as URL;
			expect(calledUrl.searchParams.get('page')).toBe('2');
			expect(calledUrl.searchParams.get('limit')).toBe('10');
			expect(calledUrl.searchParams.get('archived')).toBe('true');
		});
	});

	describe('getConversation', () => {
		it('should fetch a specific conversation', async () => {
			const mockResponse = {
				conversation: {
					id: 'conv-123',
					title: 'Test',
					messages: [],
					created_at: '2026-01-10T12:00:00Z',
					updated_at: '2026-01-10T12:00:00Z'
				},
				messages: []
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const result = await getConversation('conv-123');

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/conversations/conv-123`, {
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				}
			});
			expect(result).toEqual(mockResponse);
		});
	});

	describe('archiveConversation', () => {
		it('should archive a conversation', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({})
			});

			await archiveConversation('conv-123');

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/conversations/conv-123/archive`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				}
			});
		});

		it('should throw error on archive failure', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				json: () => Promise.resolve({ message: 'Not found' })
			});

			await expect(archiveConversation('conv-123')).rejects.toThrow('Not found');
		});
	});

	describe('deleteConversation', () => {
		it('should delete a conversation', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({})
			});

			await deleteConversation('conv-123');

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/conversations/conv-123`, {
				method: 'DELETE',
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				}
			});
		});

		it('should throw error on delete failure', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				json: () => Promise.resolve({ message: 'Forbidden' })
			});

			await expect(deleteConversation('conv-123')).rejects.toThrow('Forbidden');
		});
	});

	describe('uploadFiles', () => {
		it('should upload files and return responses', async () => {
			const mockResponse = {
				files_uploaded: 2,
				files: [
					{ name: 'file1.txt', content: 'content1', size: 100 },
					{ name: 'file2.txt', content: 'content2', size: 200 }
				]
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const files = [
				new File(['content1'], 'file1.txt'),
				new File(['content2'], 'file2.txt')
			];

			const result = await uploadFiles(files);

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/upload`, {
				method: 'POST',
				headers: {
					Authorization: 'Bearer test-token'
				},
				body: expect.any(FormData)
			});
			expect(result).toEqual(mockResponse.files);
		});

		it('should upload without auth token if not available', async () => {
			vi.mocked(localStorage.getItem).mockReturnValue(null);

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({ files_uploaded: 1, files: [] })
			});

			const files = [new File(['content'], 'test.txt')];
			await uploadFiles(files);

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/upload`, {
				method: 'POST',
				headers: {},
				body: expect.any(FormData)
			});
		});
	});

	describe('sendContextualChat', () => {
		it('should send contextual chat with context files', async () => {
			const mockResponse = {
				conversation_id: 'conv-123',
				message: {
					id: 'msg-1',
					role: 'assistant',
					content: 'Based on the context...',
					timestamp: '2026-01-10T12:00:00Z'
				}
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const request = {
				messages: [{ role: 'user' as const, content: 'Summarize this' }],
				context: ['file-id-1', 'file-id-2'],
				template: 'summary'
			};

			const result = await sendContextualChat(request);

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/chat/contextual`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				},
				body: JSON.stringify(request)
			});
			expect(result).toEqual(mockResponse);
		});
	});

	describe('getUsageStats', () => {
		it('should fetch usage statistics', async () => {
			const mockResponse = {
				total_requests: 100,
				total_tokens: 5000,
				total_cost_cents: 50,
				requests_by_model: { 'gpt-4': 50, 'gpt-3.5': 50 }
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const result = await getUsageStats();

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/usage`, {
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				}
			});
			expect(result).toEqual(mockResponse);
		});
	});

	describe('getAIInfo', () => {
		it('should fetch AI service information', async () => {
			const mockResponse = {
				provider: 'anthropic',
				templates: ['summary', 'analysis'],
				schemas: ['json', 'text'],
				streaming_supported: true,
				websocket_supported: false
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const result = await getAIInfo();

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/info`, {
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				}
			});
			expect(result).toEqual(mockResponse);
		});
	});

	describe('checkAIHealth', () => {
		it('should check AI service health', async () => {
			const mockResponse = { status: 'healthy' };

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const result = await checkAIHealth();

			expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/health`, {
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer test-token'
				}
			});
			expect(result).toEqual(mockResponse);
		});

		it('should return error message on unhealthy state', async () => {
			const mockResponse = { status: 'unhealthy', message: 'API unavailable' };

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockResponse)
			});

			const result = await checkAIHealth();
			expect(result.message).toBe('API unavailable');
		});
	});
});
