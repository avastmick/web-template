// web-template/client/src/routes/.well-known/appspecific/com.chrome.devtools.json/+server.ts

/**
 * Handle Chrome DevTools configuration requests
 *
 * Chrome DevTools automatically requests this endpoint to check for developer tools configuration.
 * By providing an empty response, we suppress the 404 errors in the logs.
 */

import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = () => {
	// Return an empty JSON object to satisfy Chrome DevTools
	// This prevents the 404 error logs while maintaining clean console output
	return json({});
};
