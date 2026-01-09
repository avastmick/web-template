import { describe, it, expect } from 'vitest';
import { validateEmail, validatePasswordStrength, validatePasswordMatch } from './validation';

describe('validateEmail', () => {
	it('should return valid for correct email format', () => {
		const result = validateEmail('test@example.com');
		expect(result.isValid).toBe(true);
		expect(result.error).toBeUndefined();
	});

	it('should return invalid for empty email', () => {
		const result = validateEmail('');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Email is required');
	});

	it('should return invalid for whitespace-only email', () => {
		const result = validateEmail('   ');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Email is required');
	});

	it('should return invalid for email without @', () => {
		const result = validateEmail('invalid-email');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Please enter a valid email address');
	});

	it('should return invalid for email without domain', () => {
		const result = validateEmail('test@');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Please enter a valid email address');
	});

	it('should return invalid for email without TLD', () => {
		const result = validateEmail('test@example');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Please enter a valid email address');
	});
});

describe('validatePasswordStrength', () => {
	it('should return valid for strong password', () => {
		const result = validatePasswordStrength('StrongPass123');
		expect(result.isValid).toBe(true);
		expect(result.error).toBeUndefined();
	});

	it('should return invalid for empty password', () => {
		const result = validatePasswordStrength('');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Password is required');
	});

	it('should return invalid for short password', () => {
		const result = validatePasswordStrength('Short1A');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Password must be at least 12 characters long');
	});

	it('should return invalid for password without lowercase', () => {
		const result = validatePasswordStrength('NOLOWERCASE123');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Password must contain at least one lowercase letter');
	});

	it('should return invalid for password without uppercase', () => {
		const result = validatePasswordStrength('nouppercase123');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Password must contain at least one uppercase letter');
	});

	it('should return invalid for password without number', () => {
		const result = validatePasswordStrength('NoNumbersHere!');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Password must contain at least one number');
	});
});

describe('validatePasswordMatch', () => {
	it('should return valid when passwords match', () => {
		const result = validatePasswordMatch('Password123', 'Password123');
		expect(result.isValid).toBe(true);
		expect(result.error).toBeUndefined();
	});

	it('should return invalid when confirm password is empty', () => {
		const result = validatePasswordMatch('Password123', '');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Please confirm your password');
	});

	it('should return invalid when passwords do not match', () => {
		const result = validatePasswordMatch('Password123', 'DifferentPass');
		expect(result.isValid).toBe(false);
		expect(result.error).toBe('Passwords do not match');
	});
});
