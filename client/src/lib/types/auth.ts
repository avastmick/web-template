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

export interface PaymentUser {
	payment_required: boolean;
	payment_status?: string;
	subscription_end_date?: string;
	has_valid_invite: boolean;
	invite_expires_at?: string;
}

export interface UnifiedAuthResponse {
	auth_token: string;
	auth_user: User;
	payment_user: PaymentUser;
}

export interface AuthState {
	user: User | null;
	token: string | null;
	isAuthenticated: boolean;
	isLoading: boolean;
	error: string | null;
	paymentUser: PaymentUser | null;
}

export interface LoginRequest {
	email: string;
	password: string;
}

export interface RegisterRequest {
	email: string;
	password: string;
}

// Keep these as aliases for backward compatibility during migration
export type RegisterResponse = UnifiedAuthResponse;
export type LoginResponse = UnifiedAuthResponse;
export type OAuthLoginResponse = UnifiedAuthResponse;

export interface AuthError {
	error: string;
}
