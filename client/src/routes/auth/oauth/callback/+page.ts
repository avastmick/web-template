import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
	// Extract OAuth parameters from URL
	const params = url.searchParams;

	return {
		token: params.get('token'),
		userId: params.get('user_id'),
		email: params.get('email'),
		error: params.get('error'),
		isNewUser: params.get('is_new_user') === 'true',
		paymentRequired: params.get('payment_required') === 'true',
		hasValidInvite: params.get('has_valid_invite') === 'true'
	};
};
