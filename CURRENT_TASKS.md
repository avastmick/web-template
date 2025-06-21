# Current Tasks

This document outlines the tasks to be completed based on `INSTRUCTIONS.md` and `PRD.md`.

## Status Summary

**Task 1.1: GitHub OAuth Integration** - **[✓] COMPLETED**
- All GitHub OAuth implementation completed successfully
- All server tests (42) and client tests (10) passing
- Code formatting and quality checks passing
- Test suite issues resolved (environment variable isolation in server tests)

**Phase 2: UI/UX Foundation** - **[✓] COMPLETED**
- Task 2.1: Foundation UI System Setup (design tokens, Tailwind CSS 4.0) - **[✓] COMPLETED**
- Task 2.2: Component Library Integration (shadcn-svelte) - **[✓] COMPLETED**
- Task 2.3: Dark/Light Mode & Advanced Theming - **[✓] COMPLETED**
- Task 2.4: Responsive Layout System - **[✓] COMPLETED**
- Task 2.5: Accessibility & WCAG Compliance - **[✓] COMPLETED**
- Task 2.6: Internationalization (i18n) Framework - **[ ] TODO**

**Phase 3: Advanced Features** - **[ ] TODO**
- Task 3.1: Generative AI Integration Framework
- Task 3.2: Stripe Payment Integration
- Task 3.3: Deployment Guides & DevOps
- Task 3.4: Template Scaffolding Tools

---

### Task 1.1: Server & Client - GitHub OAuth Integration with Invite System

This task implements GitHub OAuth as an additional authentication provider alongside the existing oAuth provider, Google.

**Key Requirements:**
- Only invited users can register or login (both local and OAuth)
- Invites are stored in a `user_invite` table with email addresses
- Registration/login attempts by non-invited users should be rejected with appropriate on-screen error messages
- Invite validation should occur before user creation in both local and OAuth flows

#### Sub-task 1.1.1: Server - Extend OAuth Models for GitHub Support
*   **Status:** **[✓] COMPLETED**
*   **Action:** Extend existing OAuth models and configuration to support GitHub as a provider
*   **Details:**
    *   Add `GitHub` variant to `OAuthProvider` enum in `server/src/models/oauth.rs`
    *   Create `GitHubUserInfo` struct following the pattern of `GoogleUserInfo`
    *   Update `OAuthConfig` in `server/src/config/oauth.rs` to support multiple providers
    *   Add GitHub OAuth environment variables: `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `GITHUB_REDIRECT_URI`
    *   Update `.envrc.example` with GitHub OAuth variables
*   **Files to Modify:**
    *   `server/src/models/oauth.rs` (add GitHub to enum and create GitHubUserInfo struct)
    *   `server/src/config/oauth.rs` (extend to support GitHub provider)
    *   `server/src/main.rs` (validate GitHub env variables on startup)
    *   `.envrc.example` (add GitHub OAuth variables)
*   **Implementation Notes:**
    *   GitHub user info endpoint: `https://api.github.com/user`
    *   GitHub OAuth authorize URL: `https://github.com/login/oauth/authorize`
    *   GitHub token URL: `https://github.com/login/oauth/access_token`
*   **Quality Checks:**
    *   `just check-server` (formatting, linting, type checking)
    *   `just build-server` (ensure compilation succeeds)


#### Sub-task 1.1.2: Server - Extend OAuth Service for GitHub
*   **Status:** **[✓] COMPLETED**
*   **Action:** Add GitHub-specific methods to the OAuth service
*   **Details:**
    *   Add `github()` method to `OAuthService` that returns GitHub OAuth client
    *   Implement `get_github_user_info()` method following `get_google_user_info()` pattern
    *   Add proper error handling for GitHub-specific API responses
    *   Ensure the service can handle both Google and GitHub providers dynamically
*   **Files to Modify:**
    *   `server/src/services/oauth_service.rs` (add GitHub methods)
*   **Implementation Notes:**
    *   GitHub requires User-Agent header for API calls
    *   GitHub returns email in a separate API call to `/user/emails` if not public
    *   Use the same `oauth2` and `reqwest` crates as Google implementation
*   **Quality Checks:**
    *   Unit tests for new GitHub methods
    *   `just check-server`
    *   `just test-server oauth_service`

#### Sub-task 1.1.3: Server - GitHub OAuth Handler Implementation
*   **Status:** **[✓] COMPLETED**
*   **Action:** Create GitHub-specific OAuth handlers following the existing Google OAuth pattern
*   **Details:**
    *   Create `github_auth()` handler for `GET /api/auth/oauth/github` - returns GitHub authorization URL
    *   Create `github_callback()` handler for `GET /api/auth/oauth/github/callback` - handles OAuth callback
    *   Follow the exact pattern of `google_auth()` and `google_callback()` handlers
    *   Ensure invite validation is performed before creating new users
    *   Reuse the existing `handle_oauth_callback()` helper function
*   **Files to Modify:**
    *   `server/src/handlers/oauth_handler.rs` (add GitHub handlers)
    *   `server/src/routes.rs` (add GitHub OAuth routes)
*   **Implementation Notes:**
    *   The handlers should be nearly identical to Google handlers, just using GitHub provider
    *   State parameter should be properly validated to prevent CSRF
    *   Error responses should match existing patterns for consistency
*   **Quality Checks:**
    *   `just check-server`
    *   Manual testing with actual GitHub OAuth app
    *   Verify invite system blocks unauthorized users

#### Sub-task 1.1.4: Client - GitHub OAuth UI Components
*   **Status:** **[✓] COMPLETED**
*   **Action:** Create GitHub Sign-In button following the existing Google OAuth pattern
*   **Details:**
    *   Create `GitHubSignInButton.svelte` component similar to existing OAuth button
    *   Add GitHub logo/icon (use SVG for consistency)
    *   Style button to match existing UI design with dark mode support
    *   Add "Sign in with GitHub" button to login and registration pages
    *   Ensure proper loading states and error handling
*   **Files to Create/Modify:**
    *   `client/src/lib/components/GitHubSignInButton.svelte` (new component)
    *   `client/src/routes/(auth)/login/+page.svelte` (add GitHub button)
    *   `client/src/routes/(auth)/register/+page.svelte` (add GitHub button)
*   **Implementation Notes:**
    *   Follow the exact component structure as Google sign-in button
    *   Use consistent spacing and styling with existing OAuth buttons
    *   Ensure accessibility (proper ARIA labels, keyboard navigation)
*   **Quality Checks:**
    *   `just check-client`
    *   Visual testing in both light and dark modes
    *   Test button states (normal, hover, disabled, loading)

#### Sub-task 1.1.5: Client - GitHub OAuth API Integration
*   **Status:** **[✓] COMPLETED**
*   **Action:** Add GitHub OAuth methods to the auth service
*   **Details:**
    *   Add `initiateGitHubOAuth()` method following the `initiateGoogleOAuth()` pattern
    *   Reuse existing `handleOAuthCallback()` method (it's provider-agnostic)
    *   No changes needed to user types (already supports provider field)
*   **Files to Modify:**
    *   `client/src/lib/services/apiAuth.ts` (add initiateGitHubOAuth method)
*   **Implementation Notes:**
    *   The method should simply redirect to `/api/auth/oauth/github`
    *   No additional types needed - existing OAuth infrastructure supports multiple providers
    *   The callback handler at `/auth/callback` already works for any provider
*   **Quality Checks:**
    *   `just check-client`
    *   Test complete OAuth flow with GitHub
    *   Verify user data is properly stored in authStore

#### Sub-task 1.1.6: End-to-End OAuth Testing
*   **Status:** **[✓] COMPLETED**
*   **Action:** Extend E2E tests to cover GitHub OAuth flow
*   **Details:**
    *   Add GitHub OAuth tests to existing test suite
    *   Test complete sign-in flow with GitHub
    *   Test registration of new users via GitHub OAuth
    *   Test invite validation (both valid and invalid invites)
    *   Test error handling for OAuth failures
*   **Files to Modify:**
    *   `client/tests/auth.test.ts` (add GitHub OAuth test cases)
*   **Implementation Notes:**
    *   Mock OAuth provider responses for consistent testing
    *   Test both success and failure scenarios
    *   Ensure tests cover the invite validation flow
*   **Quality Checks:**
    *   `just test-e2e` (run all E2E tests)
    *   All tests should pass
    *   No flaky tests

### Task 2.1: Client - Foundation UI System Setup
*   **Status:** **[✓] COMPLETED**
*   **Completion Notes:**
    *   Created comprehensive design token system with W3C DTCG standard (`design-tokens.json`)
    *   Implemented CSS custom properties for theming (`tokens.css`)
    *   Set up 8px grid spacing system and typography scale
    *   Configured sky-200/indigo-900 color scheme for light/dark themes
    *   All design tokens properly integrated with Tailwind CSS
*   **Action:** Set up modern UI foundation with Tailwind CSS 4.0 and design tokens
*   **Details:**
    *   Install and configure Tailwind CSS 4.0 with modern features
    *   Implement design token system following W3C DTCG standard
    *   Set up CSS custom properties for consistent theming
    *   Create base typography scale (4 font sizes max)
    *   Implement 8px grid spacing system
    *   Configure color system following 60-30-10 rule
*   **Files to Create/Modify:**
    *   `client/src/lib/styles/tokens.css` (design tokens)
    *   `client/src/app.css` (global styles with design system)
    *   `client/tailwind.config.js` (Tailwind 4.0 configuration)
    *   `client/src/lib/styles/design-tokens.json` (W3C DTCG token definitions)
*   **Implementation Notes:**
    *   Follow UI/UX specification in `documentation/UI-UX_SPECIFICATION.md`
    *   Implement minimalist "Form follows feeling" design philosophy
    *   Ensure mobile-first responsive design
    *   Use modern viewport units (dvh, svh) for layouts
*   **Quality Checks:**
    *   `just check-client` (formatting, linting, type checking)
    *   `just build-client` (ensure compilation succeeds)
    *   Visual regression testing for design consistency

### Task 2.2: Client - Component Library Integration
*   **Status:** **[✓] COMPLETED**
*   **Action:** Integrate shadcn-svelte UI component library
*   **Details:**
    *   Install and configure shadcn-svelte with TypeScript support
    *   Set up Bits UI as headless component foundation
    *   Create base UI components (Button, Dialog, Form, Input)
    *   Implement component composition patterns
    *   Set up component documentation and examples
    *   Ensure full accessibility compliance (WCAG 2.1 AA)
*   **Files to Create/Modify:**
    *   `client/src/lib/components/ui/` (shadcn-svelte components)
    *   `client/components.json` (shadcn-svelte configuration)
    *   `client/src/lib/components/ui/button.svelte` (button component)
    *   `client/src/lib/components/ui/dialog.svelte` (dialog component)
    *   `client/src/lib/components/ui/input.svelte` (input component)
*   **Implementation Notes:**
    *   Use shadcn-svelte copy-paste approach for zero bundle impact
    *   Ensure minimum 44px touch targets for mobile
    *   Implement proper ARIA patterns and keyboard navigation
    *   Test with screen readers (NVDA/VoiceOver)
*   **Quality Checks:**
    *   Accessibility audit with axe-core
    *   Cross-browser compatibility testing
    *   Component playground/storybook for documentation

### Task 2.3: Client - Dark/Light Mode & Advanced Theming
*   **Status:** **[✓] COMPLETED**
*   **Action:** Implement comprehensive theme system with dark/light modes
*   **Details:**
    *   Set up theme store with system preference detection
    *   Implement Tailwind CSS 4.0 custom variant for dark mode
    *   Create theme switching UI component
    *   Prevent Flash of Incorrect Theme (FOIT) in app.html
    *   Support color scheme preferences and custom theme variants
    *   Implement theme persistence with localStorage
*   **Files to Create/Modify:**
    *   `client/src/lib/stores/theme.js` (theme management store)
    *   `client/src/lib/components/ThemeToggle.svelte` (theme switcher UI)
    *   `client/src/app.html` (FOIT prevention script)
    *   `client/tailwind.config.js` (dark mode custom variants)
    *   `client/src/lib/styles/themes.css` (theme definitions)
*   **Implementation Notes:**
    *   Use `light-dark()` CSS function for automatic theme switching
    *   Implement system preference detection with `prefers-color-scheme`
    *   Support manual theme override (light/dark/system)
    *   Ensure proper color contrast ratios (4.5:1 normal, 3:1 large text)
*   **Quality Checks:**
    *   Test theme switching across all components
    *   Verify accessibility in both light and dark modes
    *   Cross-browser testing for theme persistence

### Task 2.4: Client - Responsive Layout System
*   **Status:** **[✓] COMPLETED**
*   **Action:** Implement modern responsive layout with CSS Grid and mobile-first design
*   **Details:**
    *   Create main application layout using CSS Grid
    *   Implement responsive navigation with mobile drawer
    *   Set up container query patterns for component responsiveness
    *   Create breakpoint system aligned with Tailwind CSS
    *   Implement touch-friendly interactions and gestures
    *   Add iOS safe area handling
*   **Files to Create/Modify:**
    *   `client/src/lib/components/layout/AppLayout.svelte` (main layout)
    *   `client/src/lib/components/layout/Navigation.svelte` (responsive nav)
    *   `client/src/lib/components/layout/MobileDrawer.svelte` (mobile menu)
    *   `client/src/lib/styles/layout.css` (layout utilities)
    *   `client/src/routes/+layout.svelte` (root layout implementation)
*   **Implementation Notes:**
    *   Use CSS Grid with named grid areas for semantic layouts
    *   Implement mobile-first responsive design approach
    *   Ensure 8px minimum spacing between interactive elements
    *   Use modern viewport units (dvh/svh) for full-height layouts
    *   Add swipe gestures for mobile navigation
*   **Quality Checks:**
    *   Test across all device sizes (mobile, tablet, desktop)
    *   Verify touch target sizes (minimum 44×44px)
    *   Performance testing on mobile devices

### Task 2.5: Client - Accessibility & WCAG Compliance
*   **Status:** **[✓] COMPLETED**
*   **Action:** Implement comprehensive accessibility features and WCAG 2.1 AA compliance
*   **Details:**
    *   Set up automated accessibility testing with axe-core
    *   Implement proper focus management and keyboard navigation
    *   Add ARIA patterns for complex UI components
    *   Ensure semantic HTML structure throughout
    *   Implement high contrast mode support
    *   Add screen reader testing and optimization
*   **Files to Create/Modify:**
    *   `client/src/lib/utils/accessibility.js` (a11y utilities)
    *   `client/tests/accessibility.test.js` (automated a11y tests)
    *   `client/src/lib/components/SkipToContent.svelte` (skip navigation)
    *   `client/src/lib/stores/a11y.js` (accessibility preferences)
*   **Implementation Notes:**
    *   Integrate axe-core into CI/CD pipeline
    *   Test with NVDA, VoiceOver, and JAWS screen readers
    *   Implement logical tab order throughout application
    *   Ensure all interactive elements have proper labels
    *   Support Windows High Contrast Mode
*   **Quality Checks:**
    *   Automated accessibility testing in CI
    *   Manual screen reader testing
    *   Keyboard navigation testing
    *   Color contrast verification

### Task 2.6: Client - Internationalization (i18n) Framework
*   **Status:** **[ ] TODO**
*   **Action:** Implement svelte-i18n with lazy loading and RTL support
*   **Details:**
    *   Install and configure svelte-i18n with TypeScript
    *   Set up hierarchical translation key structure
    *   Implement route-based lazy loading of translations
    *   Add RTL language support and direction switching
    *   Create translation management workflow
    *   Implement locale detection and persistence
*   **Files to Create/Modify:**
    *   `client/src/lib/stores/i18n.js` (i18n store configuration)
    *   `client/src/lib/i18n/` (translation files directory)
    *   `client/src/lib/i18n/en.json` (English translations)
    *   `client/src/lib/i18n/es.json` (Spanish translations example)
    *   `client/src/lib/components/LanguageSelector.svelte` (language picker)
*   **Implementation Notes:**
    *   Use namespace organization for efficient bundle splitting
    *   Implement lazy loading to reduce initial bundle size
    *   Support pluralization and interpolation
    *   Add date/number formatting for different locales
    *   Test with RTL languages (Arabic, Hebrew)
*   **Quality Checks:**
    *   Bundle size analysis for translation splits
    *   RTL layout testing
    *   Locale switching performance testing

### Task 3.1: Server - Generative AI Integration Framework
*   **Status:** **[ ] TODO**
*   **Action:** Create flexible framework for integrating various AI providers using the OpenRouter API (which uses the OpenAI API format)
*   **Details:**
    *   Design abstract AI provider interface with common methods
    *   Implement OpenAI GPT integration (chat completions, embeddings)
    *   Add streaming response support for real-time chat
    *   Implement token counting and cost tracking
    *   Add rate limiting and error handling with exponential backoff
*   **Files to Create/Modify:**
    *   `server/src/services/ai/` (AI provider services directory)
    *   `server/src/services/ai/provider.rs` (abstract AI provider trait to allow for later changes if required)
    *   `server/src/services/ai/openai.rs` (OpenAI implementation)
    *   `server/src/handlers/ai_handler.rs` (AI API endpoints)
    *   `server/src/models/ai.rs` (AI request/response models)
*   **Implementation Notes:**
    *   Use the `openai-api-rs` (version 6.0.6 is latest) crate for interaction with the OpenRouter API
    *   Update endpoint (see below for example)
    *   Use async streams for real-time responses
    *   Implement proper error handling for API failures
    *   Add configuration for model selection and parameters
    *   Support both text and multimodal inputs where available
    *   MUST support structured outputs with easy definition of output JSON schema
    *   MUST support easy means of prompt definition, including programmatic, conditional, composite prompt composition using `handlebars`, or similar.
*   **Quality Checks:**
    *   Unit tests
    *   Integration tests with real API responses

#### `openai-api-rs` usage

```rust
let api_key = std::env::var("OPENROUTER_API_KEY")
    .map_err(|_| anyhow::anyhow!("OPENROUTER_API_KEY not found. Fatal error."))?;

let mut client = OpenAIClient::builder()
    .with_endpoint("https://openrouter.ai/api/v1")
    .with_api_key(api_key)
    .build()?;
```

```rust

let model_name = std::env::var("MODEL_NAME")
    .map_err(|_| anyhow::anyhow!("MODEL_NAME not found. Fatal error."))?;

let req = ChatCompletionRequest::new(
    model_name,
    vec![chat_completion::ChatCompletionMessage {
        role: chat_completion::MessageRole::user,
        content: chat_completion::Content::Text(String::from("What is bitcoin?")),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    }],
);
```

### Task 3.2: Server & Client - Stripe Payment Integration
*   **Status:** **[ ] TODO**
*   **Action:** Integrate Stripe for subscription and one-time payment processing
*   **Details:**
    *   Set up Stripe webhook handling for payment events
    *   Implement subscription management (create, update, cancel)
    *   Add one-time payment processing
    *   Create payment UI components with Stripe Elements
    *   Implement billing dashboard and invoice management
    *   Add payment method management (cards, bank accounts)
    *   Handle failed payments and dunning management
*   **Files to Create/Modify:**
    *   `server/src/services/payment_service.rs` (Stripe integration)
    *   `server/src/handlers/payment_handler.rs` (payment endpoints)
    *   `server/src/models/payment.rs` (payment data models)
    *   `server/src/middleware/stripe_webhook.rs` (webhook verification)
    *   `client/src/lib/components/payments/` (payment UI components)
    *   `client/src/lib/services/payments.ts` (payment API client)
    *   `client/src/routes/billing/` (billing dashboard routes)
*   **Implementation Notes:**
    *   Use Stripe's official Rust SDK
    *   Implement idempotency for payment operations
    *   Add comprehensive webhook event handling
    *   Ensure PCI compliance best practices
*   **Quality Checks:**
    *   Test with Stripe test mode and webhook testing
    *   Security audit for payment data handling
    *   Integration tests for payment flows

### Task 3.3: Documentation - Deployment Guides
*   **Status:** **[ ] TODO**
*   **Action:** Create comprehensive deployment guides for major cloud platforms
*   **Details:**
    *   Create GCP Cloud Run deployment guide with Docker
    *   Create Vercel deployment guide for client-side
    *   Create Railway/Render deployment guide for full-stack
    *   Add database deployment options (Supabase, PlanetScale, Neon)
    *   Create CI/CD pipeline examples (GitHub Actions)
    *   Add monitoring and logging setup guides
    *   Create environment variable management guides
*   **Files to Create:**
    *   `documentation/deployment/gcp-cloud-run.md`
    *   `documentation/deployment/vercel.md`
    *   `documentation/deployment/railway.md`
    *   `documentation/deployment/database-setup.md`
    *   `documentation/deployment/ci-cd.md`
    *   `documentation/deployment/monitoring.md`
    *   `.github/workflows/` (CI/CD workflow examples)
*   **Implementation Notes:**
    *   Include Docker configurations for production
    *   Add health check endpoints for deployment platforms
    *   Document scaling considerations and performance tuning
    *   Include cost optimization strategies
*   **Quality Checks:**
    *   Test deployment guides on actual platforms
    *   Verify all environment variables and configurations
    *   Document troubleshooting common issues

### Task 3.4: Developer Experience - Template Scaffolding
*   **Status:** **[ ] TODO**
*   **Action:** Create scaffolding tools for new projects using this template
*   **Details:**
    *   Create CLI tool for project initialization
    *   Implement interactive project setup wizard
    *   Add template customization options (features to include/exclude)
    *   Create project renaming and rebranding automation
    *   Add development environment setup automation
    *   Implement feature flag system for optional components
    *   Create update mechanism for template improvements
*   **Files to Create:**
    *   `scripts/create-project.js` (project scaffolding script)
    *   `scripts/setup-wizard.js` (interactive setup)
    *   `scripts/update-template.js` (template update utility)
    *   `template.config.json` (template configuration)
    *   `documentation/template-usage.md` (usage guide)
*   **Implementation Notes:**
    *   Use Node.js for cross-platform compatibility
    *   Implement file templating with variable substitution
    *   Add Git repository initialization and cleanup
    *   Support different database options during setup
*   **Quality Checks:**
    *   Test project creation on different operating systems
    *   Verify generated projects build and run correctly
    *   Test with different configuration combinations


**Note on Testing:**
*   **Server:** Unit tests for individual functions/modules. Integration tests for API endpoints (testing request/response, database interaction). Use `cargo test`.
*   **Client:** Unit tests for Svelte components, stores, and services (using Vitest or Jest). E2E tests for user flows (using Playwright). Use `bun test`.
*   All tests should be runnable via `just` commands (e.g., `just server-test`, `just client-test`, `just test-e2e`).
