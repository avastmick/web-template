import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
	// Check for registration success message
	const registered = url.searchParams.get('registered') === 'true';

	return {
		registered
	};
};
