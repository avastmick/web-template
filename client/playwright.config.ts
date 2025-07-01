import { defineConfig } from '@playwright/test';

export default defineConfig({
	// For development, use the already running dev server
	use: {
		baseURL: 'http://localhost:8080'
	},
	// Uncomment for production testing with preview server
	// webServer: {
	// 	command: 'npm run build && npm run preview',
	// 	port: 4173
	// },
	testDir: 'e2e'
});
