import type { PageLoad } from './$types';

// Force route re-rendering in SPA mode
export const load: PageLoad = async () => {
	// This ensures the route is properly loaded and previous route components are cleaned up
	return {
		// Timestamp to force re-render
		timestamp: Date.now()
	};
};
