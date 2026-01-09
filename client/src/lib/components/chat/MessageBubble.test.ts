import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import type { ChatMessage } from '$lib/types/chat.js';

// Mock svelte-i18n
vi.mock('svelte-i18n', () => ({
	_: {
		subscribe: vi.fn((cb: (value: (key: string) => string) => void) => {
			cb((key: string) => key);
			return () => {};
		})
	}
}));

// Mock MarkdownContent component
vi.mock('./MarkdownContent.svelte', () => ({
	default: {
		render: () => ({ html: '<div>Mocked Markdown</div>' })
	}
}));

// Mock navigator.clipboard
const mockWriteText = vi.fn().mockResolvedValue(undefined);
Object.defineProperty(navigator, 'clipboard', {
	value: {
		writeText: mockWriteText
	},
	writable: true
});

import MessageBubble from './MessageBubble.svelte';

describe('MessageBubble', () => {
	const createMessage = (overrides: Partial<ChatMessage> = {}): ChatMessage => ({
		id: 'msg-123',
		role: 'user',
		content: 'Hello, world!',
		timestamp: '2026-01-10T12:00:00Z',
		...overrides
	});

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('user messages', () => {
		it('should render user message content', () => {
			const message = createMessage({ role: 'user', content: 'Test message' });
			render(MessageBubble, { props: { message } });

			expect(screen.getByText('Test message')).toBeInTheDocument();
		});

		it('should align user message to the right', () => {
			const message = createMessage({ role: 'user' });
			const { container } = render(MessageBubble, { props: { message } });

			const wrapper = container.querySelector('.justify-end');
			expect(wrapper).toBeInTheDocument();
		});

		it('should display user message with primary background', () => {
			const message = createMessage({ role: 'user' });
			const { container } = render(MessageBubble, { props: { message } });

			const messageDiv = container.querySelector('.bg-color-action-primary');
			expect(messageDiv).toBeInTheDocument();
		});

		it('should show attached files if present', () => {
			const message = createMessage({
				role: 'user',
				metadata: { attachedFiles: ['document.pdf', 'image.png'] }
			});
			render(MessageBubble, { props: { message } });

			expect(screen.getByText('document.pdf')).toBeInTheDocument();
			expect(screen.getByText('image.png')).toBeInTheDocument();
		});
	});

	describe('assistant messages', () => {
		it('should render assistant message content', () => {
			const message = createMessage({ role: 'assistant', content: 'AI response' });
			render(MessageBubble, { props: { message } });

			// Content is rendered via MarkdownContent or directly
			expect(screen.getByText('AI response')).toBeInTheDocument();
		});

		it('should align assistant message to the left', () => {
			const message = createMessage({ role: 'assistant' });
			const { container } = render(MessageBubble, { props: { message } });

			const wrapper = container.querySelector('.justify-start');
			expect(wrapper).toBeInTheDocument();
		});

		it('should show assistant avatar', () => {
			const message = createMessage({ role: 'assistant' });
			const { container } = render(MessageBubble, { props: { message } });

			// Avatar is the round div
			const avatar = container.querySelector('.rounded-full');
			expect(avatar).toBeInTheDocument();
		});

		it('should show action buttons when showActions is true', () => {
			const message = createMessage({ role: 'assistant' });
			render(MessageBubble, { props: { message, showActions: true } });

			// Action buttons have aria-labels
			expect(screen.getByLabelText('chat.message.copy')).toBeInTheDocument();
			expect(screen.getByLabelText('chat.message.regenerate')).toBeInTheDocument();
		});

		it('should hide action buttons when showActions is false', () => {
			const message = createMessage({ role: 'assistant' });
			render(MessageBubble, { props: { message, showActions: false } });

			expect(screen.queryByLabelText('chat.message.copy')).not.toBeInTheDocument();
		});
	});

	describe('streaming state', () => {
		it('should show streaming cursor when isStreaming is true', () => {
			const message = createMessage({ role: 'assistant' });
			const { container } = render(MessageBubble, {
				props: { message, isStreaming: true }
			});

			// Streaming cursor is a span with animate-pulse
			const cursor = container.querySelector('.animate-pulse');
			expect(cursor).toBeInTheDocument();
		});

		it('should hide action buttons while streaming', () => {
			const message = createMessage({ role: 'assistant' });
			render(MessageBubble, {
				props: { message, isStreaming: true, showActions: true }
			});

			expect(screen.queryByLabelText('chat.message.copy')).not.toBeInTheDocument();
		});
	});

	describe('timestamp', () => {
		it('should format and display timestamp', () => {
			const message = createMessage({
				timestamp: '2026-01-10T14:30:00Z'
			});
			const { container } = render(MessageBubble, { props: { message } });

			const time = container.querySelector('time');
			expect(time).toBeInTheDocument();
			expect(time).toHaveAttribute('datetime', '2026-01-10T14:30:00Z');
		});

		it('should handle invalid timestamp gracefully', () => {
			const message = createMessage({
				timestamp: 'invalid-date'
			});
			const { container } = render(MessageBubble, { props: { message } });

			// Should not throw, time element may be empty or not present
			const time = container.querySelector('time');
			// Either no time element or empty text
			if (time) {
				expect(time.textContent).toBe('');
			}
		});
	});

	describe('copy functionality', () => {
		it('should copy message content to clipboard on copy button click', async () => {
			const message = createMessage({
				role: 'assistant',
				content: 'Copy this text'
			});
			render(MessageBubble, { props: { message, showActions: true } });

			const copyButton = screen.getByLabelText('chat.message.copy');
			await fireEvent.click(copyButton);

			expect(mockWriteText).toHaveBeenCalledWith('Copy this text');
		});
	});

	describe('accessibility', () => {
		it('should have proper group class for hover effects', () => {
			const message = createMessage();
			const { container } = render(MessageBubble, { props: { message } });

			const group = container.querySelector('.group');
			expect(group).toBeInTheDocument();
		});

		it('should have accessible time element', () => {
			const message = createMessage();
			const { container } = render(MessageBubble, { props: { message } });

			const time = container.querySelector('time');
			if (time) {
				expect(time.tagName).toBe('TIME');
			}
		});
	});

	describe('default props', () => {
		it('should default isStreaming to false', () => {
			const message = createMessage({ role: 'assistant' });
			const { container } = render(MessageBubble, { props: { message } });

			// No streaming cursor
			const cursor = container.querySelector('.animate-pulse');
			expect(cursor).not.toBeInTheDocument();
		});

		it('should default showActions to true', () => {
			const message = createMessage({ role: 'assistant' });
			render(MessageBubble, { props: { message } });

			expect(screen.getByLabelText('chat.message.copy')).toBeInTheDocument();
		});
	});
});
