import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

// Use vi.hoisted for mock functions used in vi.mock factories
const { mockInitiateGoogleOAuth } = vi.hoisted(() => ({
	mockInitiateGoogleOAuth: vi.fn()
}));

// Mock the apiAuth module
vi.mock('$lib/services/apiAuth', () => ({
	initiateGoogleOAuth: mockInitiateGoogleOAuth
}));

import GoogleOAuthButton from './GoogleOAuthButton.svelte';

// Mock svelte-i18n
vi.mock('svelte-i18n', () => ({
	_: {
		subscribe: vi.fn((cb: (value: (key: string) => string) => void) => {
			cb((key: string) => key);
			return () => {};
		})
	}
}));

// Mock crypto.randomUUID
const mockUUID = 'test-uuid-12345';
vi.stubGlobal('crypto', {
	randomUUID: vi.fn(() => mockUUID)
});

describe('GoogleOAuthButton', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.mocked(sessionStorage.setItem).mockClear();
	});

	describe('rendering', () => {
		it('should render a button', () => {
			render(GoogleOAuthButton);
			expect(screen.getByRole('button')).toBeInTheDocument();
		});

		it('should have type="button"', () => {
			render(GoogleOAuthButton);
			const button = screen.getByRole('button');
			expect(button).toHaveAttribute('type', 'button');
		});

		it('should display Google icon', () => {
			render(GoogleOAuthButton);
			const button = screen.getByRole('button');
			const svg = button.querySelector('svg');
			expect(svg).toBeInTheDocument();
			expect(svg).toHaveAttribute('aria-hidden', 'true');
		});

		it('should display translated label', () => {
			render(GoogleOAuthButton);
			// The mock returns the key itself
			expect(screen.getByRole('button')).toHaveTextContent('auth.oauth.google');
		});

		it('should have full width styling', () => {
			render(GoogleOAuthButton);
			const button = screen.getByRole('button');
			expect(button).toHaveClass('w-full');
			expect(button).toHaveClass('justify-center');
		});
	});

	describe('props', () => {
		it('should be disabled when disabled prop is true', () => {
			render(GoogleOAuthButton, { props: { disabled: true } });
			const button = screen.getByRole('button');
			expect(button).toBeDisabled();
		});

		it('should not be disabled by default', () => {
			render(GoogleOAuthButton);
			const button = screen.getByRole('button');
			expect(button).not.toBeDisabled();
		});

		it('should apply default outline variant', () => {
			render(GoogleOAuthButton);
			const button = screen.getByRole('button');
			expect(button).toHaveClass('border');
		});

		it('should apply custom class', () => {
			render(GoogleOAuthButton, { props: { class: 'my-custom-class' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('my-custom-class');
		});
	});

	describe('OAuth flow', () => {
		it('should generate state and initiate OAuth on click', async () => {
			render(GoogleOAuthButton);
			const button = screen.getByRole('button');

			await fireEvent.click(button);

			expect(crypto.randomUUID).toHaveBeenCalled();
			expect(sessionStorage.setItem).toHaveBeenCalledWith('oauth_state', mockUUID);
			expect(mockInitiateGoogleOAuth).toHaveBeenCalledWith(mockUUID);
		});

		it('should store state in sessionStorage for CSRF protection', async () => {
			render(GoogleOAuthButton);
			const button = screen.getByRole('button');

			await fireEvent.click(button);

			expect(sessionStorage.setItem).toHaveBeenCalledWith('oauth_state', mockUUID);
		});

		it('should not initiate OAuth when disabled', async () => {
			render(GoogleOAuthButton, { props: { disabled: true } });
			const button = screen.getByRole('button');

			// Button is disabled, browser prevents click
			expect(button).toBeDisabled();
		});
	});

	describe('accessibility', () => {
		it('should have accessible icon', () => {
			render(GoogleOAuthButton);
			const button = screen.getByRole('button');
			const svg = button.querySelector('svg');
			expect(svg).toHaveAttribute('aria-hidden', 'true');
		});
	});
});
