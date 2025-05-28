// web-template/client/src/lib/types/auth.ts

/**
 * Authentication-related type definitions
 */

export interface User {
	id: string;
	email: string;
	created_at: string;
	updated_at: string;
}

export interface AuthState {
	user: User | null;
	token: string | null;
	isAuthenticated: boolean;
	isLoading: boolean;
	error: string | null;
}

export interface LoginRequest {
	email: string;
	password: string;
}

export interface RegisterRequest {
	email: string;
	password: string;
}

export interface LoginResponse {
	token: string;
	user: User;
}

export interface AuthError {
	error: string;
}
