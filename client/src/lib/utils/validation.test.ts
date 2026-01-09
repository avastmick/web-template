import { describe, expect, it } from 'vitest';

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
