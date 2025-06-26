# Current Tasks

This document outlines the tasks to be completed based on `INSTRUCTIONS.md` and `PRD.md`.

## Status Summary

### Task 1.1: Server & Client - Stripe Payment Integration
*   **Status:** **[🔄] IN PROGRESS - READY FOR IMPLEMENTATION**
*   **Action:** Integrate Stripe for subscription and one-time payment processing for users that do not have an 'invite'. Once registered, non-invited users will be presented with a Stripe payment request. All payment will be handled by Stripe, only the current status of the user's payment is needed to be held - i.e. `has_paid`, `is_current`, or similar.
*   **Current Payment Flow Issues to Fix:**
    *   OAuth registration for non-invited users shows error instead of redirecting to payment page
    *   Payment page shows Stripe initialization error (needs authentication)
    *   OAuth callback redirect should go to `/` instead of old `/profile` route
*   **Details:**
    *   Document how to integrate Stripe and which API keys, etc are required and how to get it setup
    *   Set up Stripe webhook handling for payment events
    *   Add one-time payment processing
*   **Files to Create/Modify:**
    *   `server/src/services/payment_service.rs` (Stripe integration)
    *   `server/src/handlers/payment_handler.rs` (payment endpoints)
    *   `server/src/models/payment.rs` (payment data models)
    *   `server/src/middleware/stripe_webhook.rs` (webhook verification)
    *   `client/src/lib/components/payments/` (payment UI components)
    *   `client/src/lib/services/payments.ts` (payment API client)
    *   `client/src/routes/payment/` (payment routes)
*   **Implementation Notes:**
    *   Use Stripe's official Rust SDK
    *   Implement idempotency for payment operations
    *   Add comprehensive webhook event handling
    *   Ensure PCI compliance best practices - Stripe will handle all payment, card, etc. nothing stored in application
*   **Quality Checks:**
    *   Test with Stripe test mode and webhook testing
    *   Security audit for payment data handling
    *   Integration tests for payment flows

### Task 1.2: Client - UI overhaul, optimisation and refinement
*   **Status:** **[ ] TODO
*   **Action:** Update UI on client to optimise the user experience and make all components and theming consistent
*   **Details:**
    *   Document how to integrate Stripe and which API keys, etc are required and how to get it setup
    *   Set up Stripe webhook handling for payment events
    *   Add one-time payment processing
*   **Files to Create/Modify:**
*   **Implementation Notes:**
    *   Use `tailwindcss` as means of styling. Remove all custom CSS that does not directly use `tailwindcss`.
    *   Ensure navigation bar menu is correctly reactive and scales across all screen sizes.
    *   Ensure all navigation bar components are consistent and share look and feel
    *   Ensure that all components have shared styling and theming and fully reusable
    *   Ensure that the theme is consistent across all pages; create a mechanism that allows the simple addition of pages, ensuring the new pages align with application styling, theming, etc.
*   **Quality Checks:**
    *   Use `playwright` MCP to test the UI and its consistency of look and feel
    *   Create e2e tests that ensure the UI works as expected
    *   Create a test that shows that styling and theme can be updated in one place and affect all of the client

### Task 1.3: Deployment Guides and tools
*   **Status:** **[ ] TODO**
*   **Action:** Create comprehensive deployment guides for major cloud platforms
*   **Details:**
    *   Create a PostgreSQL database provider option
    *   Create GCP Cloud Run deployment with Docker
    *   Create fly.io deployment with Docker
    *   Create CI/CD pipeline examples (GitHub Actions)
    *   Add monitoring and logging setup guides - may need refactor of server tracing to support JSON-style GCP logging
    *   Create environment variable management guides
*   **Files to Create:**
    *   `documentation/deployment/gcp-cloud-run.md`
    *   `documentation/deployment/flyio.md`
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

### Task 1.4: Developer Experience - Template Scaffolding
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
    *   `scripts/create-project.rs` (project scaffolding script)
    *   `scripts/setup-wizard.rs` (interactive setup)
    *   `scripts/update-template.rs` (template update utility)
    *   `template.config.json` (template configuration)
    *   `documentation/template-usage.md` (usage guide)
*   **Implementation Notes:**
    *   Use Cargo for cross-platform compatibility
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
