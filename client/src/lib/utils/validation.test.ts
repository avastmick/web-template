import { describe, expect, it } from 'vitest';
import fc from 'fast-check';
import { validateEmail, validatePasswordStrength, validatePasswordMatch } from './validation';

// Simple smoke test to verify vitest setup works
describe('Vitest Setup', () => {
	it('should run tests', () => {
		expect(true).toBe(true);
	});

	it('should have jsdom environment', () => {
		expect(typeof window).toBe('object');
		expect(typeof document).toBe('object');
	});

	it('should have localStorage mock', () => {
		expect(typeof localStorage).toBe('object');
		expect(typeof localStorage.getItem).toBe('function');
	});
});

// Property-based tests for validation functions
describe('Validation Property Tests', () => {
	describe('validateEmail', () => {
		it('should always return an object with isValid boolean', () => {
			fc.assert(
				fc.property(fc.string(), (email) => {
					const result = validateEmail(email);
					expect(typeof result.isValid).toBe('boolean');
					if (!result.isValid) {
						expect(typeof result.error).toBe('string');
					}
				})
			);
		});

		it('should reject all whitespace-only strings', () => {
			const whitespaceArb = fc
				.array(fc.constantFrom(' ', '\t', '\n', '\r'), { minLength: 1, maxLength: 20 })
				.map((chars) => chars.join(''));

			fc.assert(
				fc.property(whitespaceArb, (whitespace) => {
					const result = validateEmail(whitespace);
					expect(result.isValid).toBe(false);
					expect(result.error).toBe('Email is required');
				})
			);
		});

		it('should accept valid email formats', () => {
			// Generate emails in the format: localpart@domain.tld
			const alphanumeric = 'abcdefghijklmnopqrstuvwxyz0123456789';
			const localChars = 'abcdefghijklmnopqrstuvwxyz0123456789._-';
			const letters = 'abcdefghijklmnopqrstuvwxyz';

			const localPartArb = fc
				.array(fc.constantFrom(...localChars.split('')), { minLength: 1, maxLength: 20 })
				.map((chars) => chars.join(''))
				.filter((s) => !s.startsWith('.') && !s.endsWith('.') && !s.includes('..'));

			const domainArb = fc
				.array(fc.constantFrom(...alphanumeric.split('')), { minLength: 1, maxLength: 15 })
				.map((chars) => chars.join(''));

			const tldArb = fc
				.array(fc.constantFrom(...letters.split('')), { minLength: 2, maxLength: 6 })
				.map((chars) => chars.join(''));

			const validEmailArb = fc
				.tuple(localPartArb, domainArb, tldArb)
				.map(([local, domain, tld]) => `${local}@${domain}.${tld}`);

			fc.assert(
				fc.property(validEmailArb, (email) => {
					const result = validateEmail(email);
					expect(result.isValid).toBe(true);
				})
			);
		});

		it('should reject strings without @ symbol', () => {
			fc.assert(
				fc.property(
					fc.string({ minLength: 1 }).filter((s) => !s.includes('@') && s.trim().length > 0),
					(email) => {
						const result = validateEmail(email);
						expect(result.isValid).toBe(false);
					}
				)
			);
		});

		it('should be deterministic (same input always gives same output)', () => {
			fc.assert(
				fc.property(fc.string(), (email) => {
					const result1 = validateEmail(email);
					const result2 = validateEmail(email);
					expect(result1.isValid).toBe(result2.isValid);
					expect(result1.error).toBe(result2.error);
				})
			);
		});
	});

	describe('validatePasswordStrength', () => {
		it('should always return an object with isValid boolean', () => {
			fc.assert(
				fc.property(fc.string(), (password) => {
					const result = validatePasswordStrength(password);
					expect(typeof result.isValid).toBe('boolean');
					if (!result.isValid) {
						expect(typeof result.error).toBe('string');
					}
				})
			);
		});

		it('should reject passwords shorter than 12 characters', () => {
			fc.assert(
				fc.property(fc.string({ minLength: 1, maxLength: 11 }), (shortPassword) => {
					const result = validatePasswordStrength(shortPassword);
					// Should fail, either due to length or missing character types
					expect(result.isValid).toBe(false);
				})
			);
		});

		it('should accept passwords meeting all criteria', () => {
			// Generate passwords that meet all requirements
			const lowercase = 'abcdefghijklmnopqrstuvwxyz';
			const uppercase = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
			const numbers = '0123456789';
			const allChars = lowercase + uppercase + numbers;

			const lowerArb = fc
				.array(fc.constantFrom(...lowercase.split('')), { minLength: 1, maxLength: 10 })
				.map((chars) => chars.join(''));

			const upperArb = fc
				.array(fc.constantFrom(...uppercase.split('')), { minLength: 1, maxLength: 10 })
				.map((chars) => chars.join(''));

			const numArb = fc
				.array(fc.constantFrom(...numbers.split('')), { minLength: 1, maxLength: 10 })
				.map((chars) => chars.join(''));

			const extraArb = fc
				.array(fc.constantFrom(...allChars.split('')), { minLength: 0, maxLength: 20 })
				.map((chars) => chars.join(''));

			const validPasswordArb = fc
				.tuple(lowerArb, upperArb, numArb, extraArb)
				.map(([lower, upper, num, extra]) => lower + upper + num + extra)
				.filter((password) => password.length >= 12);

			fc.assert(
				fc.property(validPasswordArb, (password) => {
					const result = validatePasswordStrength(password);
					expect(result.isValid).toBe(true);
				})
			);
		});

		it('should reject passwords without lowercase letters', () => {
			const upperAndNumbers = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';

			const noLowercaseArb = fc
				.array(fc.constantFrom(...upperAndNumbers.split('')), { minLength: 12, maxLength: 30 })
				.map((chars) => chars.join(''));

			fc.assert(
				fc.property(noLowercaseArb, (noLowercase) => {
					const result = validatePasswordStrength(noLowercase);
					expect(result.isValid).toBe(false);
					expect(result.error).toBe('Password must contain at least one lowercase letter');
				})
			);
		});

		it('should reject passwords without uppercase letters', () => {
			const lowerAndNumbers = 'abcdefghijklmnopqrstuvwxyz0123456789';

			const noUppercaseArb = fc
				.array(fc.constantFrom(...lowerAndNumbers.split('')), { minLength: 12, maxLength: 30 })
				.map((chars) => chars.join(''));

			fc.assert(
				fc.property(noUppercaseArb, (noUppercase) => {
					const result = validatePasswordStrength(noUppercase);
					expect(result.isValid).toBe(false);
					expect(result.error).toBe('Password must contain at least one uppercase letter');
				})
			);
		});

		it('should reject passwords without numbers', () => {
			const lettersOnly = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';

			const noNumbersArb = fc
				.array(fc.constantFrom(...lettersOnly.split('')), { minLength: 12, maxLength: 30 })
				.map((chars) => chars.join(''));

			fc.assert(
				fc.property(noNumbersArb, (noNumbers) => {
					const result = validatePasswordStrength(noNumbers);
					expect(result.isValid).toBe(false);
					expect(result.error).toBe('Password must contain at least one number');
				})
			);
		});

		it('should be deterministic', () => {
			fc.assert(
				fc.property(fc.string(), (password) => {
					const result1 = validatePasswordStrength(password);
					const result2 = validatePasswordStrength(password);
					expect(result1.isValid).toBe(result2.isValid);
					expect(result1.error).toBe(result2.error);
				})
			);
		});
	});

	describe('validatePasswordMatch', () => {
		it('should always return an object with isValid boolean', () => {
			fc.assert(
				fc.property(fc.string(), fc.string(), (password, confirmPassword) => {
					const result = validatePasswordMatch(password, confirmPassword);
					expect(typeof result.isValid).toBe('boolean');
					if (!result.isValid) {
						expect(typeof result.error).toBe('string');
					}
				})
			);
		});

		it('should accept when passwords match', () => {
			fc.assert(
				fc.property(fc.string({ minLength: 1 }), (password) => {
					const result = validatePasswordMatch(password, password);
					expect(result.isValid).toBe(true);
				})
			);
		});

		it('should reject when passwords differ', () => {
			fc.assert(
				fc.property(fc.string({ minLength: 1 }), fc.string({ minLength: 1 }), (password, confirmPassword) => {
					fc.pre(password !== confirmPassword);
					const result = validatePasswordMatch(password, confirmPassword);
					expect(result.isValid).toBe(false);
					expect(result.error).toBe('Passwords do not match');
				})
			);
		});

		it('should reject empty confirmation password', () => {
			fc.assert(
				fc.property(fc.string({ minLength: 1 }), (password) => {
					const result = validatePasswordMatch(password, '');
					expect(result.isValid).toBe(false);
					expect(result.error).toBe('Please confirm your password');
				})
			);
		});

		it('should be symmetric for matching (order of comparison)', () => {
			fc.assert(
				fc.property(fc.string({ minLength: 1 }), (password) => {
					// If we pass the same password twice, it should always be valid
					const result = validatePasswordMatch(password, password);
					expect(result.isValid).toBe(true);
				})
			);
		});

		it('should be deterministic', () => {
			fc.assert(
				fc.property(fc.string(), fc.string(), (password, confirmPassword) => {
					const result1 = validatePasswordMatch(password, confirmPassword);
					const result2 = validatePasswordMatch(password, confirmPassword);
					expect(result1.isValid).toBe(result2.isValid);
					expect(result1.error).toBe(result2.error);
				})
			);
		});
	});
});
