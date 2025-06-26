import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: parseInt(process.env.CLIENT_PORT || '8080'),
		host: '0.0.0.0',
		proxy: {
			'/api': {
				target: process.env.SERVER_URL || 'http://localhost:8081',
				changeOrigin: true,
				secure: false
			}
		}
	},
	define: {
		// Map existing env vars to VITE_ prefixed ones for client access
		'import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY': JSON.stringify(
			process.env.STRIPE_PUBLISHABLE_KEY || ''
		)
	}
});
