/**
 * Shared validation utilities for form fields
 */

// Email validation regex (matching server-side validation)
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/**
 * Validate email format
 */
export function validateEmail(email: string): { isValid: boolean; error?: string } {
	if (!email.trim()) {
		return { isValid: false, error: 'Email is required' };
	}

	if (!EMAIL_REGEX.test(email)) {
		return { isValid: false, error: 'Please enter a valid email address' };
	}

	return { isValid: true };
}

/**
 * Validate password strength (matching server-side validation)
 */
export function validatePasswordStrength(password: string): { isValid: boolean; error?: string } {
	if (!password) {
		return { isValid: false, error: 'Password is required' };
	}

	if (password.length < 12) {
		return { isValid: false, error: 'Password must be at least 12 characters long' };
	}

	// Additional strength checks
	if (!/[a-z]/.test(password)) {
		return { isValid: false, error: 'Password must contain at least one lowercase letter' };
	}

	if (!/[A-Z]/.test(password)) {
		return { isValid: false, error: 'Password must contain at least one uppercase letter' };
	}

	if (!/[0-9]/.test(password)) {
		return { isValid: false, error: 'Password must contain at least one number' };
	}

	return { isValid: true };
}

/**
 * Validate password confirmation matches
 */
export function validatePasswordMatch(
	password: string,
	confirmPassword: string
): { isValid: boolean; error?: string } {
	if (!confirmPassword) {
		return { isValid: false, error: 'Please confirm your password' };
	}

	if (password !== confirmPassword) {
		return { isValid: false, error: 'Passwords do not match' };
	}

	return { isValid: true };
}
