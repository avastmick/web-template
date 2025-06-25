// web-template/client/src/lib/types/auth.ts

/**
 * Authentication-related type definitions
 */

export interface User {
	id: string;
	email: string;
	provider?: string;
	provider_user_id?: string;
	created_at: string;
	updated_at: string;
}

export interface AuthState {
	user: User | null;
	token: string | null;
	isAuthenticated: boolean;
	isLoading: boolean;
	error: string | null;
	paymentRequired: boolean;
}

export interface LoginRequest {
	email: string;
	password: string;
}

export interface RegisterRequest {
	email: string;
	password: string;
}

export interface RegisterResponse {
	user: User;
	payment_required: boolean;
}

export interface LoginResponse {
	token: string;
	user: User;
	payment_required: boolean;
}

export interface OAuthLoginResponse {
	token: string;
	user: User;
	is_new_user: boolean;
	payment_required?: boolean;
}

export interface AuthError {
	error: string;
}
