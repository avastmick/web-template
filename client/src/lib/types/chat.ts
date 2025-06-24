// Chat interface types for AI integration

export interface ChatMessage {
	id: string;
	role: 'user' | 'assistant' | 'system';
	content: string;
	timestamp: string;
	metadata?: Record<string, unknown>;
}

export interface Conversation {
	id: string;
	title?: string;
	messages: ChatMessage[];
	created_at: string;
	updated_at: string;
	archived_at?: string;
	model?: string;
	system_prompt?: string;
	metadata?: Record<string, unknown>;
}

export interface ChatRequest {
	messages: Array<{
		role: 'user' | 'assistant' | 'system';
		content: string;
	}>;
	model?: string;
	temperature?: number;
	max_tokens?: number;
	stream?: boolean;
	context?: string[];
	use_schema?: string;
	template?: string;
}

export interface ChatResponse {
	conversation_id: string;
	message: ChatMessage;
	usage?: {
		prompt_tokens: number;
		completion_tokens: number;
		total_tokens: number;
	};
}

export interface ConversationListResponse {
	conversations: Array<{
		id: string;
		title?: string;
		created_at: string;
		updated_at: string;
		archived_at?: string;
		message_count: number;
		last_message?: {
			role: string;
			content: string;
			timestamp: string;
		};
	}>;
	total: number;
	page?: number;
	limit?: number;
}

export interface ChatState {
	// Current conversation
	currentConversation: Conversation | null;

	// All conversations list
	conversations: ConversationListResponse['conversations'];

	// UI state
	isLoading: boolean;
	isStreaming: boolean;
	error: string | null;

	// Input state
	inputText: string;

	// File upload state
	uploadedFiles: File[];
	uploadedFileResponses: FileUploadResponse[];
	isUploading: boolean;

	// Sidebar state
	sidebarOpen: boolean;
}

export interface StreamEvent {
	type: 'start' | 'delta' | 'error' | 'done';
	id?: string;
	model?: string;
	content?: string;
	index?: number;
	finish_reason?: string;
	usage?: {
		prompt_tokens: number;
		completion_tokens: number;
		total_tokens: number;
	};
	message?: string;
}

export interface FileUploadResponse {
	name: string;
	content: string;
	mime_type?: string;
	size: number;
}

export interface UsageStats {
	total_requests: number;
	total_tokens: number;
	total_cost_cents: number;
	requests_by_model: Record<string, number>;
}

export interface AIProvider {
	name: string;
	models: string[];
	streaming_supported: boolean;
	websocket_supported: boolean;
}

export interface AIInfo {
	provider: string;
	templates: string[];
	schemas: string[];
	streaming_supported: boolean;
	websocket_supported: boolean;
}
