import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [sveltekit()],
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}'],
		environment: 'jsdom',
		// Required for Stryker mutation testing
		threads: true,
		// Coverage configuration
		coverage: {
			provider: 'v8',
			reporter: ['text', 'json', 'html'],
			include: ['src/**/*.{js,ts,svelte}'],
			exclude: [
				'src/**/*.{test,spec}.{js,ts}',
				'src/**/*.d.ts',
				'src/app.html',
				'src/hooks.server.ts'
			],
			thresholds: {
				lines: 95,
				functions: 95,
				branches: 95,
				statements: 95
			}
		},
		// Vite 6 compatibility - ensure browser conditions are used
		alias: {
			$lib: '/src/lib',
			$app: '/.svelte-kit/runtime/app'
		}
	}
});
