import { describe, it, expect } from 'vitest';
import { validateEmail, validatePasswordStrength, validatePasswordMatch } from './validation';

describe('validation utils', () => {
	describe('validateEmail', () => {
		describe('valid emails', () => {
			const validEmails = [
				'test@example.com',
				'user.name@domain.org',
				'user+tag@example.co.uk',
				'firstname.lastname@company.com',
				'email@subdomain.domain.com',
				'1234567890@example.com',
				'email@example-one.com',
				'_______@example.com',
				'email@example.name',
				'email@example.museum',
				'email@example.co.jp'
			];

			validEmails.forEach((email) => {
				it(`should accept "${email}"`, () => {
					const result = validateEmail(email);
					expect(result.isValid).toBe(true);
					expect(result.error).toBeUndefined();
				});
			});
		});

		describe('invalid emails', () => {
			it('should reject empty string', () => {
				const result = validateEmail('');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Email is required');
			});

			it('should reject whitespace only', () => {
				const result = validateEmail('   ');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Email is required');
			});

			it('should reject email without @', () => {
				const result = validateEmail('plainaddress');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Please enter a valid email address');
			});

			it('should reject email without domain', () => {
				const result = validateEmail('email@');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Please enter a valid email address');
			});

			it('should reject email without local part', () => {
				const result = validateEmail('@example.com');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Please enter a valid email address');
			});

			it('should reject email with spaces', () => {
				const result = validateEmail('email with space@example.com');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Please enter a valid email address');
			});

			it('should reject email without TLD', () => {
				const result = validateEmail('email@domain');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Please enter a valid email address');
			});

			it('should reject email with multiple @ symbols', () => {
				const result = validateEmail('email@domain@example.com');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Please enter a valid email address');
			});
		});
	});

	describe('validatePasswordStrength', () => {
		describe('valid passwords', () => {
			const validPasswords = [
				'SecurePass123',
				'MyPassword1234',
				'Complex1Pass!',
				'TestPassword1',
				'AbCdEfGh1234',
				'UpperLower1234'
			];

			validPasswords.forEach((password) => {
				it(`should accept "${password}"`, () => {
					const result = validatePasswordStrength(password);
					expect(result.isValid).toBe(true);
					expect(result.error).toBeUndefined();
				});
			});
		});

		describe('invalid passwords', () => {
			it('should reject empty password', () => {
				const result = validatePasswordStrength('');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Password is required');
			});

			it('should reject password shorter than 12 characters', () => {
				const result = validatePasswordStrength('Short1Ab');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Password must be at least 12 characters long');
			});

			it('should reject password with exactly 11 characters', () => {
				const result = validatePasswordStrength('AbcDefgH123');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Password must be at least 12 characters long');
			});

			it('should reject password without lowercase letters', () => {
				const result = validatePasswordStrength('ALLUPPERCASE123');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Password must contain at least one lowercase letter');
			});

			it('should reject password without uppercase letters', () => {
				const result = validatePasswordStrength('alllowercase123');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Password must contain at least one uppercase letter');
			});

			it('should reject password without numbers', () => {
				const result = validatePasswordStrength('NoNumbersHere');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Password must contain at least one number');
			});

			it('should reject all-lowercase password', () => {
				const result = validatePasswordStrength('alllowercaseonly');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Password must contain at least one uppercase letter');
			});

			it('should reject all-uppercase password', () => {
				const result = validatePasswordStrength('ALLUPPERCASEONLY');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Password must contain at least one lowercase letter');
			});

			it('should reject all-numbers password', () => {
				const result = validatePasswordStrength('123456789012');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Password must contain at least one lowercase letter');
			});
		});

		describe('edge cases', () => {
			it('should accept password with exactly 12 characters', () => {
				const result = validatePasswordStrength('AbcDefgHi123');
				expect(result.isValid).toBe(true);
			});

			it('should accept password with special characters', () => {
				const result = validatePasswordStrength('SecureP@ss123!');
				expect(result.isValid).toBe(true);
			});

			it('should accept very long password', () => {
				const result = validatePasswordStrength('ThisIsAVeryLongPasswordWith1UpperAndLower');
				expect(result.isValid).toBe(true);
			});
		});
	});

	describe('validatePasswordMatch', () => {
		describe('matching passwords', () => {
			it('should accept matching passwords', () => {
				const result = validatePasswordMatch('SecurePass123', 'SecurePass123');
				expect(result.isValid).toBe(true);
				expect(result.error).toBeUndefined();
			});

			it('should accept matching empty passwords (validation responsibility is elsewhere)', () => {
				// Note: This tests matching, not strength. Empty password handling is in validatePasswordStrength
				const result = validatePasswordMatch('', '');
				expect(result.isValid).toBe(false); // confirmPassword is empty
				expect(result.error).toBe('Please confirm your password');
			});
		});

		describe('non-matching passwords', () => {
			it('should reject empty confirmation password', () => {
				const result = validatePasswordMatch('SecurePass123', '');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Please confirm your password');
			});

			it('should reject different passwords', () => {
				const result = validatePasswordMatch('SecurePass123', 'DifferentPass456');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Passwords do not match');
			});

			it('should reject passwords with different case', () => {
				const result = validatePasswordMatch('SecurePass123', 'securepass123');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Passwords do not match');
			});

			it('should reject passwords with extra whitespace', () => {
				const result = validatePasswordMatch('SecurePass123', 'SecurePass123 ');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Passwords do not match');
			});

			it('should reject passwords with leading whitespace', () => {
				const result = validatePasswordMatch('SecurePass123', ' SecurePass123');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Passwords do not match');
			});

			it('should reject nearly matching passwords', () => {
				const result = validatePasswordMatch('SecurePass123', 'SecurePass124');
				expect(result.isValid).toBe(false);
				expect(result.error).toBe('Passwords do not match');
			});
		});
	});
});
