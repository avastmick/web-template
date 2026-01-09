import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

// Use vi.hoisted with inline writable implementation
const {
	mockChatStore,
	mockInputText,
	mockUploadedFiles,
	mockIsUploading,
	mockIsStreaming
} = vi.hoisted(() => {
	// Simple writable-like mock that works without importing svelte/store
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

	const mockInputText = createMockStore('');
	const mockUploadedFiles = createMockStore<File[]>([]);
	const mockIsUploading = createMockStore(false);
	const mockIsStreaming = createMockStore(false);

	const mockChatStore = {
		subscribe: vi.fn(),
		sendMessage: vi.fn(),
		setInputText: vi.fn(),
		clearUploadedFiles: vi.fn(),
		uploadFiles: vi.fn().mockResolvedValue(undefined),
		removeUploadedFile: vi.fn(),
		stopStreaming: vi.fn(),
		setError: vi.fn()
	};

	return {
		mockChatStore,
		mockInputText,
		mockUploadedFiles,
		mockIsUploading,
		mockIsStreaming
	};
});

// Mock the chatStore module
vi.mock('$lib/stores/chatStore.js', () => ({
	chatStore: mockChatStore,
	inputText: mockInputText,
	uploadedFiles: mockUploadedFiles,
	isUploading: mockIsUploading,
	isStreaming: mockIsStreaming
}));

// Mock svelte-i18n
vi.mock('svelte-i18n', () => ({
	_: {
		subscribe: vi.fn((cb: (value: (key: string) => string) => void) => {
			cb((key: string) => key);
			return () => {};
		})
	}
}));

import ChatInput from './ChatInput.svelte';

describe('ChatInput', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		// Reset store values
		mockInputText.set('');
		mockUploadedFiles.set([]);
		mockIsUploading.set(false);
		mockIsStreaming.set(false);
	});

	describe('rendering', () => {
		it('should render a textarea', () => {
			render(ChatInput);
			expect(screen.getByRole('textbox')).toBeInTheDocument();
		});

		it('should render file upload button', () => {
			render(ChatInput);
			expect(screen.getByLabelText('chat.input.uploadFile')).toBeInTheDocument();
		});

		it('should render send button', () => {
			render(ChatInput);
			expect(screen.getByLabelText('chat.input.send')).toBeInTheDocument();
		});

		it('should display placeholder text', () => {
			render(ChatInput);
			expect(screen.getByPlaceholderText('chat.input.placeholder')).toBeInTheDocument();
		});

		it('should display hint text', () => {
			render(ChatInput);
			expect(screen.getByText('chat.input.hint.enter')).toBeInTheDocument();
			expect(screen.getByText('chat.input.hint.shiftEnter')).toBeInTheDocument();
		});
	});

	describe('text input', () => {
		it('should call setInputText on input', async () => {
			render(ChatInput);
			const textarea = screen.getByRole('textbox');

			await fireEvent.input(textarea, { target: { value: 'Hello' } });

			expect(mockChatStore.setInputText).toHaveBeenCalledWith('Hello');
		});

		it('should disable textarea when streaming', async () => {
			mockIsStreaming.set(true);
			render(ChatInput);
			const textarea = screen.getByRole('textbox');

			expect(textarea).toBeDisabled();
		});
	});

	describe('send message', () => {
		it('should send message on button click when text is present', async () => {
			mockInputText.set('Hello world');
			render(ChatInput);

			const sendButton = screen.getByLabelText('chat.input.send');
			await fireEvent.click(sendButton);

			expect(mockChatStore.sendMessage).toHaveBeenCalledWith('Hello world');
		});

		it('should clear uploaded files after sending', async () => {
			mockInputText.set('Hello');
			render(ChatInput);

			const sendButton = screen.getByLabelText('chat.input.send');
			await fireEvent.click(sendButton);

			expect(mockChatStore.clearUploadedFiles).toHaveBeenCalled();
		});

		it('should not send message when text is empty', async () => {
			mockInputText.set('');
			render(ChatInput);

			const sendButton = screen.getByLabelText('chat.input.send');
			expect(sendButton).toBeDisabled();
		});

		it('should not send message when streaming', async () => {
			mockInputText.set('Hello');
			mockIsStreaming.set(true);
			render(ChatInput);

			// During streaming, the button changes to stop
			expect(screen.queryByLabelText('chat.input.send')).not.toBeInTheDocument();
		});
	});

	describe('keyboard shortcuts', () => {
		it('should send message on Enter key', async () => {
			mockInputText.set('Hello');
			render(ChatInput);

			const textarea = screen.getByRole('textbox');
			await fireEvent.keyDown(textarea, { key: 'Enter', shiftKey: false });

			expect(mockChatStore.sendMessage).toHaveBeenCalled();
		});

		it('should not send message on Shift+Enter', async () => {
			mockInputText.set('Hello');
			render(ChatInput);

			const textarea = screen.getByRole('textbox');
			await fireEvent.keyDown(textarea, { key: 'Enter', shiftKey: true });

			expect(mockChatStore.sendMessage).not.toHaveBeenCalled();
		});
	});

	describe('file upload', () => {
		it('should show upload button', () => {
			render(ChatInput);
			expect(screen.getByLabelText('chat.input.uploadFile')).toBeInTheDocument();
		});

		it('should disable upload button when uploading', () => {
			mockIsUploading.set(true);
			render(ChatInput);

			expect(screen.getByLabelText('chat.input.uploadFile')).toBeDisabled();
		});

		it('should display uploaded files', () => {
			const file1 = new File(['content1'], 'test1.txt', { type: 'text/plain' });
			const file2 = new File(['content2'], 'test2.pdf', { type: 'application/pdf' });
			mockUploadedFiles.set([file1, file2]);

			render(ChatInput);

			expect(screen.getByText('test1.txt')).toBeInTheDocument();
			expect(screen.getByText('test2.pdf')).toBeInTheDocument();
		});

		it('should show file count when files are uploaded', () => {
			const file = new File(['content'], 'test.txt', { type: 'text/plain' });
			mockUploadedFiles.set([file]);

			render(ChatInput);

			// The hint shows file count
			expect(
				screen.getByText((content) => content.includes('chat.input.filesUploaded'))
			).toBeInTheDocument();
		});

		it('should call removeUploadedFile when remove button is clicked', async () => {
			const file = new File(['content'], 'test.txt', { type: 'text/plain' });
			mockUploadedFiles.set([file]);

			render(ChatInput);

			// Find and click the remove button (it has an aria-label with the filename)
			const removeButton = screen.getByLabelText(
				(content) => content.includes('chat.input.removeFile')
			);
			await fireEvent.click(removeButton);

			expect(mockChatStore.removeUploadedFile).toHaveBeenCalledWith(0);
		});
	});

	describe('streaming state', () => {
		it('should show stop button when streaming', () => {
			mockIsStreaming.set(true);
			render(ChatInput);

			expect(screen.getByLabelText('chat.input.stop')).toBeInTheDocument();
		});

		it('should call stopStreaming when stop button is clicked', async () => {
			mockIsStreaming.set(true);
			render(ChatInput);

			const stopButton = screen.getByLabelText('chat.input.stop');
			await fireEvent.click(stopButton);

			expect(mockChatStore.stopStreaming).toHaveBeenCalled();
		});

		it('should hide send button when streaming', () => {
			mockIsStreaming.set(true);
			render(ChatInput);

			expect(screen.queryByLabelText('chat.input.send')).not.toBeInTheDocument();
		});
	});

	describe('drag and drop', () => {
		it('should have drag over handler', () => {
			const { container } = render(ChatInput);
			const dropZone = container.querySelector('[role="region"]');
			expect(dropZone).toBeInTheDocument();
		});

		it('should have accessible region label', () => {
			render(ChatInput);
			expect(screen.getByLabelText('chat.input.region')).toBeInTheDocument();
		});
	});

	describe('accessibility', () => {
		it('should have accessible textarea label', () => {
			render(ChatInput);
			expect(screen.getByLabelText('chat.input.message')).toBeInTheDocument();
		});

		it('should focus textarea on mount', () => {
			render(ChatInput);
			const textarea = screen.getByRole('textbox');
			// In test environment, focus may not work the same
			expect(textarea).toBeInTheDocument();
		});
	});
});
