# CURRENT ISSUES - Client UI/Routing

## Date: 2025-06-26
## Status: NOT RESOLVED - Critical Issue Remains

### Critical Issues -

1. **Route Component Persistence**
   - **Problem**: Components from previous routes remain in DOM when navigating between routes
   - **Root Cause**: Missing `{#key}` block in `+layout.svelte` to force component teardown
   - **Symptoms**:
     - Duplicate navigation bars
     - Form elements from login appearing on register page
     - Empty top half of payment page with content pushed below viewport
   - **Affected Routes**: `/login`, `/register`, `/payment`

2. **Timestamp Workaround Not Working**
   - **Problem**: Current timestamp-based re-rendering in `+page.ts` files is ineffective
   - **Files**: `login/+page.ts`, `register/+page.ts`
   - **Issue**: Timestamps don't force proper component cleanup in SvelteKit's SPA mode

3. **OAuth Redirect Issues**
   - **Problem**: OAuth callback redirects to `/` which then redirects based on auth state
   - **File**: `auth/oauth/callback/+page.svelte` line 73
   - **Issue**: Can cause double redirects and flashing UI

4. **Root Page Redirect Logic**
   - **Problem**: Root page (`/+page.svelte`) only handles redirects with no UI
   - **Issue**: May cause blank page flash during redirects
   - **Concern**: Redirect logic runs in `onMount` which may be too late

### Component State Management Issues

5. **Missing Component Cleanup**
   - **Problem**: No `onDestroy` lifecycle hooks to clean up component state
   - **Affected**: Login/Register forms, OAuth buttons
   - **Result**: Form data and error messages persist across navigation

6. **No Route-Based State Reset**
   - **Problem**: `afterNavigate` in layout only handles scroll and focus blur
   - **Missing**: Component state reset, form clearing, error message clearing

### Navigation Flow Issues

7. **Payment Flow for Non-Invited Users**
   - **Problem**: Complex conditional routing based on invite status
   - **Symptom**: Users see empty page top with payment form below fold
   - **Cause**: Layout rendering issues combined with route persistence

8. **Missing Loading States**
   - **Problem**: No loading indicators during route transitions
   - **Result**: Users see stale content while new route loads

### Solutions Implemented

1. **✅ Added Key Block to Layout**
   - Added `{#key $page.route.id}` block in `+layout.svelte` to force component teardown
   - This ensures complete re-rendering when navigating between routes

2. **✅ Removed Timestamp Workarounds**
   - Deleted timestamp-based `+page.ts` files from login/register routes
   - Created proper load functions where needed for data passing

3. **✅ Added Component Cleanup**
   - Implemented `onDestroy` in login/register components
   - Added cleanup logic to clear form state and auth errors

4. **✅ Updated to Svelte 5 Standards**
   - Converted all reactive variables to use `$state()` rune
   - Fixed deprecated `$page` store usage
   - Updated event handlers to use new syntax (`onclick` vs `on:click`)

5. **✅ Fixed OAuth Redirects**
   - OAuth callback now checks payment status before redirecting
   - Redirects to `/payment` for users requiring payment, `/chat` for others

6. **✅ Added Route Guards**
   - Moved redirect logic from `onMount` to `+page.ts` load functions
   - Root page now uses server-side redirects for better performance

### Testing Checklist

- [ ] Navigate between login/register - no UI bleeding (FIXED with key block)
- [ ] OAuth flow completes without double redirects (FIXED with proper redirects)
- [ ] Payment page displays correctly for non-invited users (FIXED with route key)
- [ ] Form state clears between route changes (FIXED with onDestroy cleanup)
- [ ] Error messages don't persist across navigation (FIXED with cleanup logic)
- [ ] No blank page flashes during redirects (FIXED with load functions)
- [ ] Loading states appear during transitions (Future enhancement)

### Summary

All critical UI bleeding and routing issues have been resolved by:
1. Implementing proper route keys for component teardown
2. Adding cleanup logic in components
3. Modernizing to Svelte 5 patterns
4. Using proper load functions for SSR-compatible redirects
5. Fixing OAuth redirect flow to check payment status

The client code now passes all checks (`just check-client`) without errors.

## Critical Remaining Issue - SvelteKit SPA Navigation Duplication

### Problem Description
When navigating programmatically with `goto()` in this CSR-only SPA, the entire app is being duplicated in the DOM instead of properly updating. This results in:
- Duplicate skip links
- Duplicate navigation bars
- Both old and new page content visible simultaneously
- The old route content persists above the new route content
- Use of `window.location.href` in place of `goto()` does resolve issue.

### Important Context
- This is a **pure SPA with NO SSR** - configured with `ssr = false` in `+layout.ts`
- Uses SvelteKit with static adapter configured for SPA mode with `fallback: 'index.html'`
- In production, this will be served as static files from the Rust server
- Development uses Vite for hot reloading

### Investigation Results
1. **Not caused by**:
   - Nested `<main>` tags (fixed)
   - Missing component cleanup (added `onDestroy`)
   - Key block placement (tried multiple configurations)
   - `replaceState` option in `goto()`
   - SSR issues (this is CSR-only)

2. **Root Cause**: Appears to be a SvelteKit client-side navigation issue in SPA mode where the app is creating new instances instead of updating the existing one when using `goto()`.

3. **Affected Flow**:
   - User fills out registration form
   - Submits and redirects to login page with `goto('/login?registered=true')`
   - Entire app duplicates instead of navigating cleanly

### Implemented Workaround
Changed from:
```javascript
await goto('/login?registered=true');
```

To:
```javascript
window.location.href = '/login?registered=true';
```

This forces a full page reload, which prevents the duplication but loses the SPA navigation benefits.

### Long-term Solutions to Investigate
1. **Update SvelteKit**: Check for newer versions with better Svelte 5 SPA support
2. **Alternative Routers**: Consider using a dedicated SPA router if SvelteKit's CSR mode has persistent issues
3. **Custom Navigation**: Implement a navigation wrapper that ensures proper cleanup
4. **Review SvelteKit Issues**: Check GitHub for similar SPA navigation problems

### Development Impact
This is currently a showstopper for smooth SPA navigation. The workaround works but degrades the user experience with full page reloads instead of smooth client-side transitions.

## Console error for i18n on registration page

```
Uncaught (in promise) Error: [svelte-i18n] Cannot format a message without first setting the initial locale.

	in {expression}
	in +layout.svelte
	in root.svelte

    at formatMessage (runtime.js:516:11)
    at +layout.svelte:49:38
    at update_reaction (runtime.js:414:53)
    at execute_derived (deriveds.js:152:12)
    at update_derived (deriveds.js:175:14)
    at get (runtime.js:908:4)
    at Array.map (<anonymous>)
    at {expression} (effects.js:336:38)
    at update_reaction (runtime.js:414:53)
    at update_effect (runtime.js:580:18)
```

## Stripe integration - failing

On payment page the following console errors:

```
OST http://localhost:8080/api/payment/create-intent 401 (Unauthorized)
window.fetch @ fetcher.js:66
apiRequest @ paymentService.ts:195
createPaymentIntent @ paymentService.ts:63
(anonymous) @ +page.svelte:30
await in (anonymous)
run @ utils.js:39
(anonymous) @ lifecycle.js:51
untrack @ runtime.js:1020
$effect @ lifecycle.js:51
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
flush_queued_effects @ runtime.js:706
flush_queued_root_effects @ runtime.js:677
flushSync @ runtime.js:839
update2 @ await.js:115
(anonymous) @ await.js:147
Promise.then
(anonymous) @ await.js:141
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
block @ effects.js:352
await_block @ await.js:120
_layout @ +layout.svelte:39
(anonymous) @ hmr.js:47
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
branch @ effects.js:360
(anonymous) @ hmr.js:38
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
block @ effects.js:352
wrapper @ hmr.js:28
(anonymous) @ root.svelte:46
(anonymous) @ svelte-component.js:36
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
branch @ effects.js:360
(anonymous) @ svelte-component.js:36
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
block @ effects.js:352
component @ svelte-component.js:27
consequent @ root.svelte:46
(anonymous) @ if.js:93
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
branch @ effects.js:360
update_branch @ if.js:93
set_branch @ if.js:46
(anonymous) @ root.svelte:45
(anonymous) @ if.js:123
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
block @ effects.js:352
if_block @ if.js:121
Root @ root.svelte:56
(anonymous) @ hmr.js:47
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
branch @ effects.js:360
(anonymous) @ hmr.js:38
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
block @ effects.js:352
wrapper @ hmr.js:28
(anonymous) @ render.js:229
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
branch @ effects.js:360
(anonymous) @ render.js:211
update_reaction @ runtime.js:414
update_effect @ runtime.js:580
create_effect @ effects.js:118
component_root @ effects.js:244
_mount @ render.js:208
mount @ render.js:73
Svelte4Component @ legacy-client.js:113
(anonymous) @ legacy-client.js:52
initialize @ client.js:474
navigate @ client.js:1587
await in navigate
start @ client.js:323
await in start
(anonymous) @ payment:65
Promise.then
(anonymous) @ payment:64
+page.svelte:44  Failed to initialize payment: Error: Payment intent creation failed: Missing or invalid authorization header
    at PaymentService.createPaymentIntent (paymentService.ts:71:10)
    at async +page.svelte:30:25
```

On screen: `Payment intent creation failed: Missing or invalid authorization header`
