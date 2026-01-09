import { svelteTesting } from '@testing-library/svelte/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [sveltekit(), svelteTesting()],
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}'],
		exclude: ['e2e/**', 'node_modules/**'],
		environment: 'jsdom',
		setupFiles: ['./vitest.setup.ts'],
		globals: true,
		coverage: {
			provider: 'v8',
			reporter: ['text', 'json', 'html'],
			reportsDirectory: './coverage',
			include: ['src/**/*.{js,ts,svelte}'],
			exclude: [
				'src/**/*.d.ts',
				'src/**/*.test.{js,ts}',
				'src/**/*.spec.{js,ts}',
				'src/app.html'
			]
		}
	}
});
