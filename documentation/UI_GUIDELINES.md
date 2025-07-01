# UI Guidelines

## Overview

This document defines the UI/UX standards and theming system for the web-template project. All components and pages MUST follow these guidelines to ensure consistency across the application.

## Table of Contents

1. [Theme System Architecture](#theme-system-architecture)
2. [Color System](#color-system)
3. [Typography](#typography)
4. [Spacing and Layout](#spacing-and-layout)
5. [Component Guidelines](#component-guidelines)
6. [Dark/Light Mode](#darklight-mode)
7. [Responsive Design](#responsive-design)
8. [Accessibility](#accessibility)
9. [Implementation Guidelines](#implementation-guidelines)
10. [Testing Checklist](#testing-checklist)

## Theme System Architecture

### Core Files

1. **Design Tokens**: `/client/src/lib/styles/tokens.css`
   - Defines all CSS custom properties (variables)
   - Core color palette, typography, spacing, shadows, transitions
   - Base values that themes build upon

2. **Theme Variants**: `/client/src/lib/styles/themes.css`
   - Light/dark theme definitions
   - Maps semantic colors to design tokens

3. **Tailwind Configuration**: `/client/tailwind.config.js`
   - Maps CSS variables to Tailwind utilities
   - Extends default theme with custom values
   - Configures dark mode as `class`-based

4. **Theme Store**: `/client/src/lib/stores/theme.ts`
   - Manages theme state (light/dark/system)
   - Handles localStorage persistence
   - Updates DOM classes

5. **FOIT Prevention**: `/client/src/app.html`
   - Inline script applies saved theme before render
   - Prevents flash of incorrect theme

### Technology Stack

- **Frontend**: Svelte 5 + SvelteKit (SPA mode)
- **Styling**: Tailwind CSS with custom design tokens
- **State Management**: Svelte stores
- **i18n**: Custom implementation with lazy loading
- **Build Tool**: Vite
- **Testing**: Vitest + Playwright

## Color System

### Design Philosophy

The project uses a Tailwind-based color system with indigo as the primary color palette and amber for accents:
- **Background**: Indigo scale (indigo-950 for dark, indigo-50 for light)
- **Accent**: Amber for highlighting (focus borders, hover states)
- **Text**: High contrast for readability
- **Semantic colors**: Status indicators (success, error, warning, info)

### Color Tokens

**NEVER hardcode colors. Always use CSS variables through Tailwind classes.**

#### Background Colors
- `bg-bg-primary` / `bg-background-primary` - Main background
- `bg-bg-secondary` / `bg-background-secondary` - Secondary surfaces
- `bg-bg-tertiary` / `bg-background-tertiary` - Tertiary surfaces
- `bg-bg-elevated` / `bg-background-elevated` - Elevated surfaces (modals, dropdowns)

#### Text Colors
- `text-text-primary` - Primary text
- `text-text-secondary` - Secondary/muted text
- `text-text-tertiary` - Tertiary/disabled text
- `text-text-on-primary` - Text on primary colored backgrounds

#### Border Colors
- `border-border-default` - Default borders
- `border-border-strong` - Emphasized borders
- `border-border-subtle` - Subtle borders

#### Status Colors
- Success: `bg-color-success-background`, `text-color-success`
- Warning: `bg-color-warning-background`, `text-color-warning`
- Error: `bg-color-error-background`, `text-color-error`
- Info: `bg-color-info-background`, `text-color-info`

## Typography

### Font Families
- Sans: `font-sans` - System font stack for UI
- Mono: `font-mono` - Monospace for code

### Font Sizes
Use Tailwind's default scale:
- `text-xs` - 12px
- `text-sm` - 14px
- `text-base` - 16px (body text)
- `text-lg` - 18px
- `text-xl` - 20px
- `text-2xl` - 24px (headings)
- `text-3xl` - 30px (page titles)

### Font Weights
- Normal text: `font-normal` (400)
- Emphasized: `font-medium` (500)
- Headers: `font-semibold` (600) or `font-bold` (700)

## Spacing and Layout

### Spacing Scale
Use Tailwind's 8px-based spacing scale consistently:
- `p-1` (4px) to `p-12` (48px)
- Common patterns: `p-4`, `px-6 py-4`, `gap-4`
- Use the 8px grid: 4px, 8px, 16px, 24px, 32px, 48px

### Container Widths
- Full width: `w-full`
- Constrained: `max-w-7xl mx-auto`
- Reading width: `max-w-prose`
- Form width: `max-w-md`

## Component Guidelines

### Reusable UI Components

The project includes a set of reusable UI components in `/client/src/lib/components/ui/`:

1. **Button** (`button.svelte`)
   - Variants: default, primary, secondary, outline, ghost, destructive
   - Sizes: sm, md, lg
   - Supports loading and disabled states

2. **Input** (`input.svelte`)
   - Includes error state styling
   - Consistent focus rings
   - Support for all HTML input types

3. **Alert** (`alert.svelte`)
   - Variants: info, success, warning, error
   - Used for all status messages
   - Consistent styling across the app

4. **Card** (`card.svelte`)
   - Consistent container styling
   - Elevated background with borders
   - Proper padding and shadows

5. **FormField** (`form-field.svelte`)
   - Combines label, input, and error message
   - Reduces code duplication
   - Ensures consistent form layouts

6. **Container** (`container.svelte`)
   - Responsive width constraints
   - Centered content with padding

7. **Grid** (`grid.svelte`) & **Flex** (`flex.svelte`)
   - Layout utilities with responsive options
   - Consistent gap and alignment options

### Usage Examples

```svelte
<!-- Button -->
<Button variant="primary" size="lg">
  Click me
</Button>

<!-- Alert -->
<Alert variant="error">
  {errorMessage}
</Alert>

<!-- FormField -->
<FormField
  label="Email"
  error={errors.email}
>
  <Input
    type="email"
    bind:value={email}
    error={!!errors.email}
  />
</FormField>

<!-- Card -->
<Card>
  <h2 class="text-xl font-semibold text-text-primary mb-4">Card Title</h2>
  <p class="text-text-secondary">Card content</p>
</Card>
```

## Dark/Light Mode

### Implementation
1. Theme toggle updates the store: `$theme = 'light' | 'dark' | 'system'`
2. Store applies class to `<html>` element
3. CSS variables change based on `.light` or `.dark` class
4. All colors automatically update

### Best Practices
- Never use `dark:` prefixes in Tailwind - use semantic color tokens
- Always test both themes during development
- Ensure sufficient contrast in both modes

### Theme Values
The theme follows a logical inversion pattern:
- Dark mode: indigo-950 background → indigo-50 for light mode
- Text colors invert for proper contrast
- Accent colors (amber) remain consistent

## Responsive Design

### Breakpoints
Use Tailwind's mobile-first approach:
- `sm:` - 640px and up
- `md:` - 768px and up
- `lg:` - 1024px and up
- `xl:` - 1280px and up
- `2xl:` - 1536px and up

### Layout Patterns
```css
/* Mobile-first grid that adapts to larger screens */
class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"

/* Responsive padding */
class="p-4 md:p-6 lg:p-8"

/* Responsive text */
class="text-base md:text-lg lg:text-xl"
```

### Touch Targets
- Minimum 44×44px for all interactive elements
- Use `min-h-[44px] min-w-[44px]` for custom elements
- 8px minimum spacing between touch targets

## Accessibility

### Core Requirements
- **Color Contrast**: 4.5:1 for normal text, 3:1 for large text
- **Keyboard Navigation**: All functionality accessible via keyboard
- **Screen Reader Support**: Proper semantic HTML and ARIA attributes
- **Focus Management**: Clear focus indicators using amber accent color

### Implementation Patterns
```svelte
<!-- Accessible button with loading state -->
<button
  aria-busy={loading}
  aria-disabled={disabled}
  disabled={disabled}
>
  {#if loading}
    <span class="sr-only">Loading...</span>
    <Spinner />
  {/if}
  {text}
</button>

<!-- Accessible form field -->
<FormField
  label="Email"
  error={errors.email}
  required
>
  <Input
    type="email"
    bind:value={email}
    error={!!errors.email}
    aria-invalid={!!errors.email}
    aria-describedby={errors.email ? 'email-error' : undefined}
  />
</FormField>
```

## Implementation Guidelines

### For New Components

1. **Always check existing components first** for patterns to follow
2. **Use only theme-aware classes**:
   ```svelte
   <!-- ❌ BAD -->
   <div class="bg-gray-800 text-white dark:bg-white dark:text-black">

   <!-- ✅ GOOD -->
   <div class="bg-bg-primary text-text-primary">
   ```

3. **Follow component structure**:
   ```svelte
   <script lang="ts">
     // Props and logic
   </script>

   <!-- Component markup -->

   <style>
     /* Only if absolutely necessary - prefer Tailwind */
   </style>
   ```

### Common Mistakes to Avoid

#### ❌ DON'T

1. **Hardcode colors**
   ```html
   <div class="bg-indigo-900 text-white">
   ```

2. **Use dark: prefixes**
   ```html
   <div class="bg-white dark:bg-gray-800">
   ```

3. **Create custom CSS when Tailwind utilities exist**
   ```css
   .custom-padding { padding: 16px; } /* Use p-4 instead */
   ```

4. **Forget hover/focus states**
   ```html
   <button class="bg-color-primary">Click</button>
   ```

#### ✅ DO

1. **Use semantic tokens**
   ```html
   <div class="bg-bg-primary text-text-primary">
   ```

2. **Include interactive states**
   ```html
   <button class="bg-color-primary hover:bg-color-primary-hover focus:ring-2 focus:ring-amber-500">
   ```

3. **Use existing components**
   ```svelte
   <Button variant="primary">Click</Button>
   ```

4. **Test in both themes**

## Testing Checklist

Before committing any UI changes:

- [ ] Component works in light mode
- [ ] Component works in dark mode
- [ ] No hardcoded colors
- [ ] Uses existing UI components where applicable
- [ ] Follows spacing guidelines (8px grid)
- [ ] Responsive on mobile/tablet/desktop
- [ ] Accessible (proper contrast, focus states, ARIA labels)
- [ ] Keyboard navigable
- [ ] Touch targets are at least 44×44px
- [ ] Consistent with existing components
- [ ] Passes `just check-client`
- [ ] E2E tests pass

## File Size Limits

All component files must be under 600 lines. If a component exceeds this:
1. Extract sub-components into separate files
2. Move complex logic to utility functions
3. Consider splitting into multiple smaller components

## References

- [Tailwind Configuration](/client/tailwind.config.js)
- [Design Tokens](/client/src/lib/styles/tokens.css)
- [Theme Variants](/client/src/lib/styles/themes.css)
- [Theme Store](/client/src/lib/stores/theme.ts)
- [Architecture Documentation](./ARCHITECTURE.md#user-interface)
- [UI Components](/client/src/lib/components/ui/)
