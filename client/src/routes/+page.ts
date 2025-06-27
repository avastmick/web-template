import type { PageLoad } from './$types';
import { checkAuthAndRedirect } from '$lib/utils/auth';

export const load: PageLoad = async ({ url }) => {
	// Use centralized auth check to handle redirects
	// This will automatically redirect to the appropriate page
	// Pass the current URL to avoid accessing page store in load context
	await checkAuthAndRedirect({ currentUrl: url });

	// Return empty object as this page just handles redirects
	return {};
};
