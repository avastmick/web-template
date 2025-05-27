# WEB-TEMPLATE CODEBASE GUIDE

The web-template is a project that will enable the quick-start for a high-performance web application. The project goals are:

- Performance: the project must meet or exceed the very highest performance expectations in all cases
- Secure: the project must be provably secure, using the latest cryptographic techniques and best practices in all cases
- High quality: the code must be provably high-quality, with all static analysis and code quality check turned to 11!
- Lightweight: we need to ensure that the application uses the smallest possible memory and storage footprint, both on the server and in the browser
- Easy to use: usage of web application, the developer experience, and operation of the application should be first class
- Beautiful: the web application must be first class in look and feel, engaging users, causing them to ooh and aah in use.
- Functional: the web application in particular must be able to leverage modern web application tools, extensions and widgets easily, to make development fast and easy.

Always read `documentation/PRD.md` and the project `README.md`.

## Overview

There are two components to this project:

- `client`, which is a `svelte/svelte-kit` application that serves the web application
- `server`, which is a `rust/axum` application that offers `REST` API endpoints and interacts with a database using `sqlx`

The database is configurable, we will use `SQLite` as a starting point. Database migrations will be managed using `dbmate` locally.

We will use `just` to wrap commands to make development and deployment easy.
We will use `overmind` to run the servers locally.

## Documentation and additional context

There is additional documentation context available in `context/`. The following is currently available:

- `svelte` docs - TODO

Ask the user to add the link to needed SDK or API documentation and `git clone`, or `cuRl`/`wget` the resource locally so it can be read. Always ask before attempting to download.

## Package Management

This project uses Bun and Cargo exclusively. Do not use `npm` or `node`, etc. commands. ALWAYS use `bun add` or `cargo add`: DO NOT update configuration files directly

### Package Commands

#### Client - using Bun

- Install packages: `bun add <package-name>` (Note: have user run this command instead of Claude)
- Remove packages: `bun remove <package-name>`
- Check for unused dependencies: `bun pm ls --prod=false` (identifies unused dependencies)
- Clean unused packages: `bun pm prune` (removes unused dependencies)

#### Server - using Cargo

TODO: complete instructions here


## Project Commands

- Build: `bun run build` (vite build)
- Lint: `bun run lint` (prettier --check . && eslint .)
- Format: `bun run format` (prettier --write .)
- Basic Type Check: `bun run check` (basic svelte-kit check)
- Strict Type Check: `bun run check:strict` (uses strict TypeScript configuration, default)
- Test (all): `bun run test` (runs e2e tests)
- Test (single): `bun playwright test <path-to-test-file>`
- Just: `just <command>` (see justfile for available commands)

## Just Commands

- `just`: List all available commands
TODO: Add detailed compilation, linting, and style checks for both client and server. Ensure both client and server are strict, as per `clippy`-level checking

## Code Style Guidelines

- Use TypeScript for type safety
- Indentation: Tabs (not spaces)
- Quotes: Single quotes
- Line width: 140 characters max
- No trailing commas
- Svelte components use PascalCase (.svelte files)
- Variables/functions use camelCase
- Prefer async/await over promise chains
- Use descriptive error messages in catch blocks
- Import order: external libraries first, then internal modules

## Code Quality Standards

- Adhere to general quality standards:
  - Accuracy: Does the code do what it's supposed to?
  - Correctness: Are there any bugs?
  - Efficiency: Does the code execute tasks without wasting resources?
  - Maintainability: How easy is it to update and modify?
  - Readability: Is the code easy to understand?
  - Security: How well does the code protect against vulnerabilities?
- Apply the DRY principle (Don't Repeat Yourself) to avoid inconsistencies:
  - Extract common code into shared utilities/services (like our authFetch utility)
  - Reuse functionality between components and handlers
  - Keep authentication and validation logic consistent across endpoints
  - Inconsistent implementations of similar functionality lead to bugs
- Avoid deep nesting, recursion and other overly complex coding blocks
- Refactor code into as simple a form as possible after it has been functionally proven
- Include tests for functions that use algorithms or complex calculations
- Ensure functions are concise and handle a single task, and that they are suitably commented to provide tooling (e.g. LSP) input and usage.
- Ensure files are less than 500 lines, and hold code that is conducting similar, or related tasks: DO NOT mix functional code in one source file
- Ensure that all source files contain suitable comment heading that summarises the file contents and its usage, in a way that will be useful to an LSP and other code editor tooling.
- Avoid TypeScript non-null assertions (!) where possible:
  - Instead of `const item = array[0]!` use a check like `if (array.length > 0) const item = array[0]`
  - Always handle the potential absence of data explicitly
  - Use proper TypeScript patterns like optional chaining or type guards

## TypeScript Strictness Settings

This project uses a very strict TypeScript configuration with settings similar to Rust's compiler checks:

- `strict`: true (enables all strict type checking options)
- `noImplicitAny`: true (raise error on expressions and declarations with an implied 'any' type)
- `strictNullChecks`: true (enable strict null checking)
- `noImplicitReturns`: true (ensure every code path returns a value for functions)
- `noUnusedLocals` & `noUnusedParameters`: true (report errors on unused local variables and parameters)
- `noUncheckedIndexedAccess`: true (add undefined to index signatures)
- `exactOptionalPropertyTypes`: true (disable looser handling of undefined in optional properties)

Always handle nullability explicitly and ensure all code paths are covered. This prevents most runtime errors.

## Environment Variables

- ALL ENV variables including ALL secrets are held in `.envrc` ONLY (managed by direnv)
- `.envrc` is included in `.gitignore` to prevent committing secrets
- Required ENV variables:
  - DATABASE_URL
  - JWT_SECRET
- Optional ENV variables for model merging:
  - OPENAI_API_KEY
  - GEMINI_API_KEY
  - MISTRAL_API_KEY
- Use uppercase for all environment variable names

## Pre-commit Hooks

- Pre-commit hooks are set up to:
  - Check for secrets using gitleaks
  - Run formatting
  - Run linting
  - Run type checking

## Project Structure

TODO: SveltKit client with Rust server (REST API)

## Presentation Guidelines

- Keep presentation markdown (slides.md) as plain text where possible
- For styling elements (quotes, lists, etc.), add CSS in the index.html file
- Use standard markdown syntax in slides.md
- Never use inline HTML or CSS in the markdown files
- Add all custom styling to the index.html file in the style section
- Maintain separation of content (markdown) and presentation (CSS)

## Important Notes

- Never attempt to commit code, only test commit using dry-run:
  - `git commit --dry-run -m "message"` to verify what would be committed
- Never attempt to run the development server with `just dev` or `bun run dev`:
  - Claude cannot see the server output in the terminal
  - Let the user run the server themselves
- Always run `bun run build` after making changes to verify they build successfully:
  - This ensures changes will pass the pre-commit build check
  - Building should be done before suggesting commits
- Do not modify existing, working code:
  - Only make changes specifically requested in the requirements
  - If you encounter potential issues in existing code, bring them to the user's attention rather than modifying them directly
  - Focus on implementing new functionality as specified without refactoring existing code unless explicitly requested
