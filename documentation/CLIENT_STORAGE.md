# Client Storage Documentation

## Overview

This document describes how the web-template client application uses browser storage (localStorage) to persist data. All developers working on the client should reference this to ensure consistent storage key usage and avoid conflicts.

## Storage Keys Reference

### Authentication Keys

| Key | Value Type | Description | Used By | Set By |
|-----|------------|-------------|---------|---------|
| `auth_token` | `string` | JWT authentication token | All authenticated API requests | `authStore.loginSuccess()` |
| `auth_user` | `JSON string` | Serialized user object | Auth state management | `authStore.loginSuccess()`, `authStore.updateUser()` |
| `payment_required` | `"true"` or absent | Flag indicating user needs to complete payment | Payment flow routing | `authStore.setPaymentRequired()` |

### Theme Keys

| Key | Value Type | Description | Used By | Set By |
|-----|------------|-------------|---------|---------|
| `theme` | `"light"` \| `"dark"` | User's selected theme preference | Theme system | `theme.ts` store |

### Locale Keys

| Key | Value Type | Description | Used By | Set By |
|-----|------------|-------------|---------|---------|
| `preferred-locale` | `string` | User's selected language code (e.g., "en-US", "es-ES") | i18n system | `locale.ts` store |

### Chat Keys

| Key | Value Type | Description | Used By | Set By |
|-----|------------|-------------|---------|---------|
| `chat_conversations` | `JSON string` | Serialized array of conversation metadata | Chat interface | `chatStore` |
| `chat_active_conversation` | `string` | ID of the currently active conversation | Chat interface | `chatStore` |

## Implementation Details

### Auth Storage (`authStore.ts`)

The auth store manages two localStorage keys:

```typescript
const TOKEN_STORAGE_KEY = 'auth_token';
const USER_STORAGE_KEY = 'auth_user';
```

The payment store manages one sessionStorage key:

```typescript
const PAYMENT_REQUIRED_KEY = 'payment_user';
```

**Important:** Any service that needs to access the auth token MUST use the key `'auth_token'`, not variations like `'authToken'` or `'auth-token'`.

### Accessing Auth Token

There are two recommended ways to access the auth token:

1. **Via authStore subscription (Preferred for reactive contexts):**
```typescript
import { authStore } from '$lib/stores';

// Subscribe to get current token
let currentToken: string | null = null;
const unsubscribe = authStore.subscribe((state) => {
    currentToken = state.token;
});
unsubscribe(); // Immediately unsubscribe after getting value
```

2. **Direct localStorage access (For simple, non-reactive contexts):**
```typescript
const token = localStorage.getItem('auth_token'); // Note: exact key name!
if (token) {
    headers.Authorization = `Bearer ${token}`;
}
```

### Storage Initialization

The auth store is initialized in the root layout (`+layout.svelte`) on mount:

```typescript
onMount(() => {
    authStore.init(); // Loads stored auth data from localStorage
});
```

This ensures auth state is restored when the user refreshes the page or returns to the app.

### Storage Cleanup

When a user logs out, all auth-related storage is cleared:

```typescript
logout: () => {
    if (browser) {
        localStorage.removeItem(TOKEN_STORAGE_KEY);
        localStorage.removeItem(USER_STORAGE_KEY);
        sessionStorage.removeItem(PAYMENT_REQUIRED_KEY);
    }
    set(initialState);
}
```

## Best Practices

1. **Always use exact key names** - Storage keys are case-sensitive and must match exactly
2. **Check for browser environment** - Always wrap localStorage access in `if (browser)` checks when in SSR contexts
3. **Handle corrupted data** - Always wrap JSON.parse in try-catch blocks when reading stored JSON
4. **Clear on logout** - Ensure all sensitive data is removed from storage on logout
5. **Document new keys** - If you add new localStorage keys, update this document

## Common Pitfalls

1. **Key name mismatch** - Using `'authToken'` instead of `'auth_token'`
2. **Missing browser check** - Accessing localStorage during SSR will throw errors
3. **Forgetting to stringify/parse** - localStorage only stores strings, not objects
4. **Not handling null** - localStorage.getItem returns `null` for missing keys, not `undefined`

## Security Considerations

1. **JWT tokens are sensitive** - Never log or expose auth tokens in console/UI
2. **Clear on logout** - Always remove auth data when user logs out
3. **Validate on init** - The auth store validates stored tokens on initialization
4. **No secrets in storage** - Never store API keys, secrets, or sensitive configuration

## Adding New Storage Keys

If you need to add new localStorage keys:

1. Define the key as a constant in the relevant store/service
2. Use a descriptive, consistent naming pattern (snake_case preferred)
3. Document the key in this file
4. Ensure cleanup on logout if auth-related
5. Handle missing/corrupted data gracefully

## Environment Variables

The client uses Vite for bundling, which requires environment variables to be prefixed with `VITE_` to be accessible in the browser. However, to avoid duplication, the Vite config maps existing server-side environment variables to VITE_ prefixed ones.

### Required Environment Variables

| Variable | Description | Example | Mapped to |
|----------|-------------|---------|-----------|
| `VITE_SERVER_PORT` | Port where the API server runs | `8081` | Used directly |
| `STRIPE_PUBLISHABLE_KEY` | Stripe publishable key for payments | `pk_test_...` | `VITE_STRIPE_PUBLISHABLE_KEY` |

### Environment Variable Mapping

The `vite.config.ts` file maps non-VITE prefixed variables for client use:

```typescript
define: {
  'import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY': JSON.stringify(process.env.STRIPE_PUBLISHABLE_KEY || '')
}
```

### Accessing Environment Variables

```typescript
// In client code, always use VITE_ prefix
const stripeKey = import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY;

// The actual env var is STRIPE_PUBLISHABLE_KEY (without VITE_)
// But Vite maps it to VITE_STRIPE_PUBLISHABLE_KEY for client access
```

## Migration Strategy

If storage keys need to be changed:

1. Support both old and new keys during transition
2. Migrate data on first read
3. Clean up old keys after migration
4. Update this documentation
5. Notify team of breaking changes

## Testing Storage

When testing storage-dependent features:

1. Clear localStorage between tests
2. Mock localStorage for unit tests
3. Test corruption scenarios
4. Test missing data scenarios
5. Verify cleanup on logout
