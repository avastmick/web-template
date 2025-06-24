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
  - See `documentation/UI_UX_THEME.md` for theming guidelines and implementation details
- Task 2.4: Responsive Layout System - **[✓] COMPLETED**
- Task 2.5: Accessibility & WCAG Compliance - **[✓] COMPLETED**
- Task 2.6: Internationalization (i18n) Framework - **[✓] COMPLETED**
- Task 2.7: Complete i18n Implementation Across All Pages - **[✓] COMPLETED**

**Phase 3: Advanced Features** - **[🔄] IN PROGRESS**
- Task 3.1: Generative AI Integration Framework - **[🔄] INTEGRATION TESTS COMPLETE - CLIENT UI NEEDED**
- Task 3.2: Stripe Payment Integration - **[ ] TODO**
- Task 3.3: Deployment Guides & DevOps - **[ ] TODO**
- Task 3.4: Template Scaffolding Tools - **[ ] TODO**

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
*   **Status:** **[✓] COMPLETED**
*   **Completion Notes:**
    *   Successfully implemented svelte-i18n with TypeScript support
    *   Created comprehensive i18n infrastructure with 4 languages (en-US, es-ES, zh-CN, ar-SA)
    *   Implemented lazy loading for translation bundles with code splitting
    *   Added locale store with localStorage persistence and browser detection
    *   Created LanguageSelector component with accessibility features
    *   Implemented RTL support with automatic direction switching for Arabic
    *   Complete translation coverage for all UI components (navigation, forms, buttons, etc.)
    *   Written comprehensive Playwright E2E test suite (12/12 tests passing)
    *   Fixed UI styling issues (input field shadows, language selector styling)
    *   All client checks, builds, and tests passing
    *   **Note:** Framework is complete but some pages still need i18n implementation (see Task 2.7)

### Task 2.7: Client - Complete i18n Implementation Across All Pages
*   **Status:** **[✓] COMPLETED**
*   **Completion Notes:**
    *   Fixed profile page (`/profile/+page.svelte`) - converted 20+ hardcoded strings to translation keys
    *   Fixed OAuth callback page (`/auth/oauth/callback/+page.svelte`) - converted 15+ hardcoded strings
    *   Fixed profile layout (`/profile/+layout.svelte`) - converted authentication messages
    *   Fixed main layout (`/+layout.svelte`) - converted loading and accessibility messages
    *   All pages now properly use i18n translation keys throughout
    *   Created comprehensive i18n validation E2E tests that detect hardcoded text
    *   All translation keys exist in all 4 locale files (en-US, es-ES, zh-CN, ar-SA)
    *   Tests confirm no hardcoded English text remains on any page
    *   Full internationalization compliance achieved across the entire application
*   **Action:** Complete internationalization implementation for all remaining pages and components
*   **Priority:** High - Required for production readiness
*   **Discovered Issues:**
    *   Profile page (`/profile/+page.svelte`) has 20+ hardcoded English strings
    *   OAuth callback page (`/auth/oauth/callback/+page.svelte`) has 15+ hardcoded strings
    *   Profile layout (`/profile/+layout.svelte`) has authentication messages
    *   Main layout (`/+layout.svelte`) has loading and accessibility messages
*   **Detailed Sub-tasks:**
    1.  **Fix Profile Page i18n Implementation**
        *   Add `import { _ } from 'svelte-i18n';` to profile page
        *   Replace hardcoded page title and meta description
        *   Convert all user-facing text to translation keys
        *   Update profile information section, status section, actions
    2.  **Fix OAuth Callback Page i18n Implementation**
        *   Add i18n import and page meta translations
        *   Convert all authentication state messages
        *   Translate error messages for all OAuth failure scenarios
        *   Update success and loading messages
    3.  **Fix Profile Layout i18n Implementation**
        *   Add i18n import for authentication checking message
        *   Ensure proper translation key usage
    4.  **Fix Main Layout i18n Implementation**
        *   Convert loading message and accessibility skip links
        *   Ensure all user-facing text uses translation keys
    5.  **Update Documentation and Process**
        *   Created comprehensive i18n documentation (`client/src/lib/i18n/README.md`)
        *   Created implementation process guide (`documentation/I18N_PROCESS.md`)
        *   Created i18n validation E2E tests (`e2e/i18n-validation.test.ts`)
    6.  **Run Comprehensive i18n Validation**
        *   Execute automated i18n compliance tests
        *   Manually test all pages in all 4 languages
        *   Verify RTL layout for Arabic on all pages
        *   Ensure page titles and meta descriptions are translated
*   **Files to Modify:**
    *   `client/src/routes/profile/+page.svelte` (convert 20+ hardcoded strings)
    *   `client/src/routes/auth/oauth/callback/+page.svelte` (convert 15+ hardcoded strings)
    *   `client/src/routes/profile/+layout.svelte` (convert auth messages)
    *   `client/src/routes/+layout.svelte` (convert loading/accessibility messages)
*   **Translation Keys Added:**
    *   Added 40+ new translation keys to all 4 locale files
    *   Profile page keys: `profile.pageTitle`, `profile.welcome`, `profile.userInfo.*`, etc.
    *   Auth flow keys: `auth.processing`, `auth.oauth.*`, `auth.errors.*`, etc.
    *   Accessibility keys: `accessibility.skipToMain`, `accessibility.skipToNav`
*   **Quality Checks:**
    *   All translation keys exist in all 4 locale files (en-US, es-ES, zh-CN, ar-SA)
    *   i18n validation E2E tests pass
    *   Manual testing in all languages
    *   RTL layout verification for Arabic
    *   Page title and meta description translation verification
*   **Action:** Implement svelte-i18n with lazy loading and RTL support
*   **Architecture Decisions:**
    *   Default locale: `en-US`
    *   Translation key structure: Flat structure for lightweight frontend (e.g., `"login.button.text"`)
    *   Locale persistence: localStorage only (no cookies, no URL prefixes)
    *   URL structure: Keep URLs clean without locale prefixes
*   **Detailed Sub-tasks:**
    1.  **Research and design i18n architecture for SvelteKit with svelte-i18n**
        *   Analyze svelte-i18n documentation and best practices
        *   Design flat translation key structure for efficiency
        *   Plan lazy loading strategy for translation bundles
    2.  **Install and configure svelte-i18n with TypeScript support**
        *   Add svelte-i18n dependency
        *   Configure TypeScript types for translations
        *   Set up build-time optimizations
    3.  **Create i18n store with locale detection and persistence**
        *   Implement locale store with localStorage persistence
        *   Add browser language detection fallback
        *   Handle SSR/hydration considerations
    4.  **Set up translation file structure and lazy loading**
        *   Create flat JSON structure for translations
        *   Implement dynamic import for translation bundles
        *   Configure Vite for optimal code splitting
    5.  **Implement LanguageSelector component**
        *   Create accessible dropdown/menu component
        *   Add language display names in native languages
        *   Integrate with theme system for consistent styling
    6.  **Add RTL support and direction switching**
        *   Implement `dir` attribute switching on `<html>`
        *   Add RTL-aware CSS utilities
        *   Test with RTL languages (Arabic, Hebrew)
    7.  **Create translation keys for existing UI components**
        *   Audit all user-facing text in components
        *   Extract strings to translation files
        *   Implement translation function usage
    8.  **Write unit tests for i18n functionality**
        *   Test locale switching and persistence
        *   Test translation loading and fallbacks
        *   Test RTL direction switching
    9.  **Write Playwright E2E tests for language switching**
        *   Test complete language switching flow
        *   Verify persistence across page reloads
        *   Test RTL layout rendering
*   **Files to Create/Modify:**
    *   `client/src/lib/stores/i18n.ts` (i18n store configuration with TypeScript)
    *   `client/src/lib/i18n/` (translation files directory)
    *   `client/src/lib/i18n/en-US.json` (English US translations)
    *   `client/src/lib/i18n/es-ES.json` (Spanish translations example)
    *   `client/src/lib/i18n/zh-CN.json` (Chinese translations example)
    *   `client/src/lib/i18n/ar-SA.json` (Arabic translations for RTL testing)
    *   `client/src/lib/components/LanguageSelector.svelte` (language picker)
    *   `client/src/lib/utils/i18n.ts` (i18n utilities and helpers)
*   **Implementation Notes:**
    *   Use flat key structure: `"auth.login.title"`, `"auth.login.submit"`
    *   Implement lazy loading to reduce initial bundle size
    *   Support pluralization and interpolation
    *   Add date/number formatting for different locales
    *   Ensure proper TypeScript types for translation keys
*   **Quality Checks:**
    *   Bundle size analysis for translation splits
    *   RTL layout testing with Arabic/Hebrew
    *   Locale switching performance testing
    *   Accessibility testing for language selector
    *   Unit tests: `just test-client i18n`
    *   E2E tests: `just test-e2e language`

### Task 3.1: Server - Generative AI Integration Framework
*   **Status:** **[🔄] INTEGRATION TESTS COMPLETE - CLIENT UI NEEDED**
*   **Completion Notes:**
    *   ✅ AI service architecture and provider abstraction implemented
    *   ✅ OpenRouter API integration with OpenAI format complete
    *   ✅ Comprehensive AI endpoints created (chat, streaming, file upload, conversation management)
    *   ✅ Database persistence for conversations, messages, and usage tracking
    *   ✅ Template system with Handlebars for prompt management
    *   ✅ Token counting and cost estimation functionality
    *   ✅ Error handling and authentication integration
    *   ✅ All dead code eliminated and quality checks passing
    *   ✅ **COMPLETED: Comprehensive integration test suite with verbose logging and real AI testing support**
    *   🔄 **NEXT: Implement client-side chat interface (Task 3.1.12)**
*   **Action:** Create flexible framework for integrating various AI providers using the OpenRouter API (which uses the OpenAI API format)
*   **Architecture Decisions:**
    *   Models: Use models defined in `.envrc.example` (via environment variables)
    *   Prompt organization: Feature-specific paths for reusability
    *   Chat persistence: Store conversations per-user in database
    *   Rate limiting: Per-user configurable limits
    *   API design: WebSockets/SSE for real-time streaming (future: voice support)
*   **Detailed Sub-tasks:**
    1.  **Design AI service architecture and provider abstraction**
        *   Define provider trait for multiple AI backends
        *   Design modular service structure
        *   Plan WebSocket/SSE streaming architecture
    2.  **Add OpenRouter/OpenAI dependencies to server**
        *   Add `openai-api-rs` v6.0.6 to Cargo.toml
        *   Add `handlebars` for templating
        *   Add `serde_json` and schema dependencies
    3.  **Implement AI provider trait and OpenRouter provider**
        *   Create abstract `AiProvider` trait
        *   Implement `OpenRouterProvider` with OpenAI API format
        *   Configure endpoint and authentication
    4.  **Create prompt template system with Handlebars**
        *   Set up Handlebars template engine
        *   Create feature-specific template directories
        *   Implement template composition and partials
    5.  **Implement structured output with JSON schemas**
        *   Create composable schema components
        *   Implement schema assembly for requests
        *   Add response validation against schemas
    6.  **Add streaming support for AI responses**
        *   Implement SSE endpoint for streaming
        *   Add WebSocket support for bidirectional communication
        *   Handle stream interruption and reconnection
    7.  **Implement token counting and cost tracking**
        *   Add token counting utilities
        *   Track usage per user and model
        *   Store cost data in database
    8.  **Create AI chat API endpoints**
        *   Implement REST endpoint for single completions
        *   Add SSE endpoint for streaming responses
        *   Create WebSocket handler for chat sessions
    9.  **Add rate limiting and error handling**
        *   Implement per-user rate limiting (configurable)
        *   Add exponential backoff for API failures
        *   Create comprehensive error types
    10. **Write integration tests for AI service**
        *   Test provider abstraction
        *   Test streaming functionality
        *   Test rate limiting and error scenarios
    11. **Write comprehensive integration tests for AI chat functionality**
        *   **Simple chat request/response test**
            - Send basic chat message via `/api/ai/chat` endpoint
            - Verify response structure and AI-generated content
            - Test with different models (controlled by `AI_DEFAULT_MODEL` env var)
            - Validate database persistence of conversation and messages
            - Verify token counting and usage tracking
        *   **Chat with uploaded document context test**
            - Upload text file via `/api/ai/upload` endpoint
            - Send chat request with file context via `/api/ai/chat/contextual`
            - Verify AI response incorporates document context
            - Test multiple file uploads and context merging
            - Validate file content is properly parsed and included
        *   **Chat archival and retrieval test**
            - Create conversation with multiple messages
            - Archive conversation via `/api/ai/conversations/:id/archive`
            - Verify conversation marked as archived in database
            - Test conversation retrieval with archived status
            - Validate archived conversations don't appear in active list
        *   **Chat history persistence and reuse test**
            - Create conversation with initial context
            - Send multiple messages to build conversation history
            - Retrieve conversation via `/api/ai/conversations/:id`
            - Send new message to existing conversation
            - Verify AI response uses full conversation context
            - Test context window management for long conversations
        *   **Model switching via environment variable test**
            - Test with `AI_DEFAULT_MODEL=gpt-4o-mini` (default)
            - Change env var to `AI_DEFAULT_MODEL=claude-3-haiku-20240307`
            - Restart service and verify new model is used
            - Test model-specific response differences
            - Verify usage tracking records correct model
            - Test model override in request payload
        *   **Error handling and edge cases test**
            - Test invalid API keys and provider failures
            - Test rate limiting enforcement
            - Test malformed requests and validation
            - Test conversation not found scenarios
            - Test file upload size limits and invalid formats
        *   **Streaming and real-time features test**
            - Test SSE streaming via `/api/ai/chat/stream`
            - Verify incremental response chunks
            - Test stream interruption and cleanup
            - Validate real-time typing indicators
        *   **Authentication and authorization test**
            - Test JWT token validation on all AI endpoints
            - Test unauthorized access rejection
            - Test user isolation (users can't access others' conversations)
            - Test admin functionality (invite management, usage stats)
    12. **✅ COMPLETED: Create comprehensive integration test suite for AI chat functionality**
        *   ✅ Implemented 17 comprehensive integration tests covering all AI endpoints and functionality
        *   ✅ Added verbose logging mode with detailed AI interaction visibility (`--verbose` flag)
        *   ✅ Added real API key testing support for live AI provider testing (`--real-ai` flag)
        *   ✅ Enhanced `justfile` with `just test-integration ai_integration_tests --verbose --real-ai`
        *   ✅ Tests cover: chat flow, file upload, conversation management, error handling, streaming, auth
        *   ✅ Database validation for conversation/message persistence and usage tracking
        *   ✅ All 17 tests passing with comprehensive error handling and edge case coverage
*   **Integration Test Files to Create:**
    *   `server/tests/integration/ai_chat_tests.rs` (comprehensive AI chat integration tests)
    *   `server/tests/integration/ai_upload_tests.rs` (file upload and context tests)
    *   `server/tests/integration/ai_conversation_tests.rs` (conversation persistence tests)
    *   `server/tests/integration/ai_model_switching_tests.rs` (model configuration tests)
    *   `server/tests/integration/ai_auth_tests.rs` (authentication and authorization tests)
    *   `server/tests/integration/ai_streaming_tests.rs` (SSE streaming tests)
    *   `client/tests/e2e/ai-chat-flow.test.ts` (end-to-end chat functionality)
    *   `client/tests/e2e/ai-file-upload.test.ts` (file upload UI flow)
    *   `client/tests/e2e/ai-conversation-management.test.ts` (conversation UI management)

*   **Test Environment Configuration:**
    *   Add `AI_DEFAULT_MODEL=gpt-4o` to test environment
    *   Set up test database with AI-related tables
    *   Configure test API keys for OpenRouter (use test/mock endpoints)
    *   Add test file uploads directory with sample documents
    *   Configure JWT test tokens for authentication tests

*   **Files to Create/Modify:**
    Align to the following sourcecode layout:
    ```
    server/src/services/ai/
    ├── assistant/
    │   └── mod.rs         # Lifecycle management, orchestration
    ├── context/
    │   └── mod.rs         # Context assembly, caching
    ├── prompt/
    │   ├── mod.rs         # Handlebars engine, template loading
    │   ├── templates/     # Feature-specific templates
    │   │   ├── chat/      # Chat-specific prompts
    │   │   └── common/    # Shared prompt partials
    │   └── schema/
    │       ├── mod.rs     # Schema assembly
    │       └── components/# Reusable schema parts
    └── provider/
        ├── mod.rs         # Provider trait definition
        └── openrouter.rs  # OpenRouter implementation
    ```

    Plus the following to enable the client interaction:
    *   `server/src/handlers/ai_handler.rs` (REST/SSE/WebSocket endpoints)
    *   `server/src/models/ai.rs` (request/response models)
    *   `server/src/db/chat.rs` (chat persistence)
    *   `client/src/lib/services/ai.ts` (AI service client)
    *   `client/src/lib/components/Chat.svelte` (chat UI component)
    *   `client/src/lib/stores/chat.ts` (chat state management)
*   **Implementation Notes:**
    *   Use `openai-api-rs` v6.0.6 with custom endpoint:
        ```rust
        let client = OpenAIClient::builder()
            .with_endpoint("https://openrouter.ai/api/v1")
            .with_api_key(api_key)
            .build()?;
        ```
    *   Environment variables: `OPENROUTER_API_KEY`, `MODEL_NAME`
    *   All AI interactions use structured JSON outputs
    *   Handlebars templates support conditionals and partials
    *   WebSocket/SSE for streaming (prepare for voice in future)
    *   Per-user rate limits stored in database
*   **Quality Checks:**
    *   Unit tests for all service modules
    *   Integration tests with mocked API responses
    *   Load testing for rate limiting
    *   E2E tests for complete chat flow
    *   Security audit for prompt injection
    *   Performance testing for streaming

#### `openai-api-rs` usage

See [openai-api-rs](https://github.com/dongri/openai-api-rs)

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

### Task 3.1.12: Client - Chat Interface Integration and Main Page Redesign
*   **Status:** **[ ] TODO**
*   **Priority:** **HIGH** - Next immediate task per INSTRUCTIONS.md
*   **Action:** Replace current home page with elegant chat interface using ChatGPT.com as UI/UX benchmark
*   **Details:**
    *   Remove current home page content and replace with chat interface
    *   Create elegant chat panel with conversation history sidebar (left panel)
    *   Implement file upload functionality for document context
    *   Support real-time streaming chat responses
    *   Add conversation management (new, archive, delete)
    *   Integrate with existing AI service endpoints
    *   Ensure responsive design for mobile/tablet/desktop
    *   Add typing indicators and message states
    *   Implement proper authentication flow integration
*   **UI/UX Requirements:**
    *   Use ChatGPT.com as benchmark for layout and user experience
    *   Left sidebar: Conversation history with search and organization
    *   Main area: Chat messages with clean bubble design
    *   Input area: Text input with file upload and send button
    *   Support for markdown rendering in AI responses
    *   Dark/light mode integration with existing theme system
    *   Mobile-responsive with collapsible sidebar
*   **Technical Requirements:**
    *   Integrate with existing AI service endpoints (`/api/ai/chat`, `/api/ai/conversations`, etc.)
    *   Implement streaming responses using SSE (`/api/ai/chat/stream`)
    *   Add file upload using `/api/ai/upload` endpoint
    *   Use existing i18n system for all user-facing text
    *   Follow accessibility guidelines (WCAG 2.1 AA)
    *   Implement proper error handling and loading states
*   **Files to Create/Modify:**
    *   `client/src/routes/+page.svelte` (main chat interface - replace existing content)
    *   `client/src/lib/components/chat/ChatInterface.svelte` (main chat component)
    *   `client/src/lib/components/chat/ConversationSidebar.svelte` (conversation history)
    *   `client/src/lib/components/chat/MessageBubble.svelte` (individual message display)
    *   `client/src/lib/components/chat/ChatInput.svelte` (message input with file upload)
    *   `client/src/lib/components/chat/FileUpload.svelte` (drag-drop file upload)
    *   `client/src/lib/services/aiClient.ts` (AI API client service)
    *   `client/src/lib/stores/chatStore.ts` (chat state management)
    *   `client/src/lib/types/chat.ts` (TypeScript types for chat)
*   **Playwright Research Task:**
    *   Use Playwright to capture ChatGPT.com layout and interaction patterns
    *   Document key UI/UX elements and behaviors
    *   Create design specification based on captured patterns
*   **Implementation Notes:**
    *   Only show chat interface when user is authenticated (redirect to login if not)
    *   Use SvelteKit's `+layout.svelte` for authentication guard
    *   Implement real-time updates using Server-Sent Events (SSE)
    *   Support file drag-and-drop with visual feedback
    *   Add message copying, regeneration, and export features
    *   Implement conversation search and filtering
*   **Quality Checks:**
    *   All existing quality checks must pass (`just check`)
    *   E2E tests for complete chat flow
    *   Accessibility testing with screen readers
    *   Performance testing for large conversation histories
    *   Mobile responsiveness testing
    *   File upload testing with various file types

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
    *   Create a PostgreSQL database provider option
    *   Create GCP Cloud Run deployment guide with Docker
    *   Create fly.io deployment guide with Docker
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
