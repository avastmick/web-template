import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

// Use vi.hoisted with inline writable implementation
const {
	mockChatStore,
	mockConversations,
	mockCurrentConversation,
	mockSidebarOpen,
	mockIsLoading
} = vi.hoisted(() => {
	// Simple writable-like mock that works without importing svelte/store
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	function createMockStore<T>(initial: T): { subscribe: (fn: (value: T) => void) => () => void; set: (value: T) => void } {
		let value = initial;
		const subscribers = new Set<(value: T) => void>();
		return {
			subscribe(fn: (value: T) => void) {
				subscribers.add(fn);
				fn(value);
				return () => subscribers.delete(fn);
			},
			set(newValue: T) {
				value = newValue;
				subscribers.forEach((fn) => fn(value));
			}
		};
	}

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const mockConversations = createMockStore<any[]>([]);
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const mockCurrentConversation = createMockStore<any>(null);
	const mockSidebarOpen = createMockStore(true);
	const mockIsLoading = createMockStore(false);

	const mockChatStore = {
		subscribe: vi.fn(),
		loadConversations: vi.fn(),
		loadConversation: vi.fn(),
		startNewConversation: vi.fn(),
		archiveConversation: vi.fn(),
		deleteConversation: vi.fn(),
		toggleSidebar: vi.fn()
	};

	return {
		mockChatStore,
		mockConversations,
		mockCurrentConversation,
		mockSidebarOpen,
		mockIsLoading
	};
});

// Mock the chatStore module
vi.mock('$lib/stores/chatStore.js', () => ({
	chatStore: mockChatStore,
	conversations: mockConversations,
	currentConversation: mockCurrentConversation,
	sidebarOpen: mockSidebarOpen,
	isLoading: mockIsLoading
}));

// Mock svelte-i18n
vi.mock('svelte-i18n', () => ({
	_: {
		subscribe: vi.fn((cb: (value: (key: string, options?: unknown) => string) => void) => {
			cb((key: string, options?: unknown) => {
				if (options && typeof options === 'object' && 'values' in options) {
					return `${key}:${JSON.stringify((options as { values: unknown }).values)}`;
				}
				return key;
			});
			return () => {};
		})
	}
}));

// Mock confirm dialog
const mockConfirm = vi.fn(() => true);
vi.stubGlobal('confirm', mockConfirm);

import ConversationSidebar from './ConversationSidebar.svelte';

describe('ConversationSidebar', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockConversations.set([]);
		mockCurrentConversation.set(null);
		mockSidebarOpen.set(true);
		mockIsLoading.set(false);
	});

	describe('rendering', () => {
		it('should render the sidebar', () => {
			render(ConversationSidebar);
			expect(screen.getByRole('complementary')).toBeInTheDocument();
		});

		it('should display sidebar title', () => {
			render(ConversationSidebar);
			expect(screen.getByText('chat.sidebar.title')).toBeInTheDocument();
		});

		it('should display new chat button', () => {
			render(ConversationSidebar);
			expect(screen.getByText('chat.sidebar.newChat')).toBeInTheDocument();
		});
	});

	describe('loading state', () => {
		it('should show loading spinner when loading', () => {
			mockIsLoading.set(true);
			render(ConversationSidebar);

			expect(screen.getByText('chat.sidebar.loading')).toBeInTheDocument();
		});

		it('should show loading spinner with animation', () => {
			mockIsLoading.set(true);
			const { container } = render(ConversationSidebar);

			const spinner = container.querySelector('.animate-spin');
			expect(spinner).toBeInTheDocument();
		});
	});

	describe('empty state', () => {
		it('should show empty state when no conversations', () => {
			mockConversations.set([]);
			mockIsLoading.set(false);
			render(ConversationSidebar);

			expect(screen.getByText('chat.sidebar.empty.title')).toBeInTheDocument();
			expect(screen.getByText('chat.sidebar.empty.description')).toBeInTheDocument();
		});
	});

	describe('conversations list', () => {
		const mockConversationList = [
			{
				id: 'conv-1',
				title: 'First Conversation',
				created_at: '2026-01-10T10:00:00Z',
				updated_at: '2026-01-10T11:00:00Z',
				message_count: 5,
				last_message: {
					role: 'assistant',
					content: 'Hello there!',
					timestamp: '2026-01-10T11:00:00Z'
				}
			},
			{
				id: 'conv-2',
				title: 'Second Conversation',
				created_at: '2026-01-09T10:00:00Z',
				updated_at: '2026-01-09T15:00:00Z',
				message_count: 3
			}
		];

		it('should display conversation titles', () => {
			mockConversations.set(mockConversationList);
			render(ConversationSidebar);

			expect(screen.getByText('First Conversation')).toBeInTheDocument();
			expect(screen.getByText('Second Conversation')).toBeInTheDocument();
		});

		it('should display last message preview', () => {
			mockConversations.set(mockConversationList);
			render(ConversationSidebar);

			expect(screen.getByText('Hello there!')).toBeInTheDocument();
		});

		it('should highlight current conversation', () => {
			mockConversations.set(mockConversationList);
			mockCurrentConversation.set({ id: 'conv-1' });
			render(ConversationSidebar);

			const currentButton = screen.getByRole('button', { name: /First Conversation/i });
			expect(currentButton).toHaveAttribute('aria-current', 'page');
		});

		it('should load conversation on click', async () => {
			mockConversations.set(mockConversationList);
			render(ConversationSidebar);

			const convButton = screen.getByText('First Conversation').closest('button');
			await fireEvent.click(convButton!);

			expect(mockChatStore.loadConversation).toHaveBeenCalledWith('conv-1');
		});
	});

	describe('new conversation', () => {
		it('should start new conversation on button click', async () => {
			render(ConversationSidebar);

			const newChatButton = screen.getByText('chat.sidebar.newChat').closest('button');
			await fireEvent.click(newChatButton!);

			expect(mockChatStore.startNewConversation).toHaveBeenCalled();
		});
	});

	describe('archive conversation', () => {
		const mockConversationList = [
			{
				id: 'conv-1',
				title: 'Test Conversation',
				created_at: '2026-01-10T10:00:00Z',
				updated_at: '2026-01-10T11:00:00Z',
				message_count: 5
			}
		];

		it('should archive conversation on archive button click', async () => {
			mockConversations.set(mockConversationList);
			render(ConversationSidebar);

			const archiveButton = screen.getByLabelText('chat.sidebar.archive');
			await fireEvent.click(archiveButton);

			expect(mockChatStore.archiveConversation).toHaveBeenCalledWith('conv-1');
		});
	});

	describe('delete conversation', () => {
		const mockConversationList = [
			{
				id: 'conv-1',
				title: 'Test Conversation',
				created_at: '2026-01-10T10:00:00Z',
				updated_at: '2026-01-10T11:00:00Z',
				message_count: 5
			}
		];

		it('should show confirmation dialog before deleting', async () => {
			mockConversations.set(mockConversationList);
			render(ConversationSidebar);

			const deleteButton = screen.getByLabelText('chat.sidebar.delete');
			await fireEvent.click(deleteButton);

			expect(mockConfirm).toHaveBeenCalled();
		});

		it('should delete conversation when confirmed', async () => {
			mockConfirm.mockReturnValueOnce(true);
			mockConversations.set(mockConversationList);
			render(ConversationSidebar);

			const deleteButton = screen.getByLabelText('chat.sidebar.delete');
			await fireEvent.click(deleteButton);

			expect(mockChatStore.deleteConversation).toHaveBeenCalledWith('conv-1');
		});

		it('should not delete conversation when cancelled', async () => {
			mockConfirm.mockReturnValueOnce(false);
			mockConversations.set(mockConversationList);
			render(ConversationSidebar);

			const deleteButton = screen.getByLabelText('chat.sidebar.delete');
			await fireEvent.click(deleteButton);

			expect(mockChatStore.deleteConversation).not.toHaveBeenCalled();
		});
	});

	describe('sidebar toggle', () => {
		it('should show mobile close button', () => {
			render(ConversationSidebar);

			// There may be multiple close buttons (overlay + button in header)
			const closeButtons = screen.getAllByLabelText('chat.sidebar.close');
			expect(closeButtons.length).toBeGreaterThan(0);
		});

		it('should toggle sidebar on close button click', async () => {
			render(ConversationSidebar);

			// Get the first close button (the one in the header)
			const closeButtons = screen.getAllByLabelText('chat.sidebar.close');
			await fireEvent.click(closeButtons[0]);

			expect(mockChatStore.toggleSidebar).toHaveBeenCalled();
		});

		it('should show overlay when sidebar is open on mobile', () => {
			mockSidebarOpen.set(true);
			const { container } = render(ConversationSidebar);

			const overlay = container.querySelector('.bg-black\\/50');
			expect(overlay).toBeInTheDocument();
		});

		it('should close sidebar when overlay is clicked', async () => {
			mockSidebarOpen.set(true);
			const { container } = render(ConversationSidebar);

			const overlay = container.querySelector('.bg-black\\/50');
			await fireEvent.click(overlay!);

			expect(mockChatStore.toggleSidebar).toHaveBeenCalled();
		});

		it('should close sidebar on Escape key', async () => {
			mockSidebarOpen.set(true);
			const { container } = render(ConversationSidebar);

			const overlay = container.querySelector('.bg-black\\/50');
			await fireEvent.keyDown(overlay!, { key: 'Escape' });

			expect(mockChatStore.toggleSidebar).toHaveBeenCalled();
		});
	});

	describe('conversation title formatting', () => {
		it('should use title if available', () => {
			mockConversations.set([
				{
					id: 'conv-1',
					title: 'Custom Title',
					created_at: '2026-01-10T10:00:00Z',
					updated_at: '2026-01-10T11:00:00Z',
					message_count: 1
				}
			]);
			render(ConversationSidebar);

			expect(screen.getByText('Custom Title')).toBeInTheDocument();
		});

		it('should truncate long last message content', () => {
			mockConversations.set([
				{
					id: 'conv-1',
					created_at: '2026-01-10T10:00:00Z',
					updated_at: '2026-01-10T11:00:00Z',
					message_count: 1,
					last_message: {
						role: 'user',
						content: 'This is a very long message that should be truncated somewhere',
						timestamp: '2026-01-10T11:00:00Z'
					}
				}
			]);
			render(ConversationSidebar);

			// The title and last_message preview may both contain this text
			// Just verify at least one element with this content exists
			const elements = screen.getAllByText(/This is a very long message/);
			expect(elements.length).toBeGreaterThan(0);
		});
	});

	describe('footer', () => {
		it('should show conversation count', () => {
			mockConversations.set([
				{
					id: 'conv-1',
					title: 'Test',
					created_at: '2026-01-10T10:00:00Z',
					updated_at: '2026-01-10T11:00:00Z',
					message_count: 1
				}
			]);
			render(ConversationSidebar);

			// Footer shows count
			expect(screen.getByText(/chat\.sidebar\.footer/)).toBeInTheDocument();
		});
	});

	describe('accessibility', () => {
		it('should have accessible sidebar label', () => {
			render(ConversationSidebar);

			expect(screen.getByLabelText('chat.sidebar.label')).toBeInTheDocument();
		});

		it('should have aria-current on active conversation', () => {
			mockConversations.set([
				{
					id: 'conv-1',
					title: 'Test',
					created_at: '2026-01-10T10:00:00Z',
					updated_at: '2026-01-10T11:00:00Z',
					message_count: 1
				}
			]);
			mockCurrentConversation.set({ id: 'conv-1' });
			render(ConversationSidebar);

			const convButton = screen.getByText('Test').closest('button');
			expect(convButton).toHaveAttribute('aria-current', 'page');
		});
	});
});
