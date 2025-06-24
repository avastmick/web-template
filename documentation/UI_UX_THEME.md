# UI/UX and Theme Documentation

## Overview

This document defines the UI/UX standards and theming system for the web-template project. All components and pages MUST follow these guidelines to ensure consistency across the application.

## Table of Contents

1. [Theme System Architecture](#theme-system-architecture)
2. [Color System](#color-system)
3. [Typography](#typography)
4. [Spacing and Layout](#spacing-and-layout)
5. [Component Guidelines](#component-guidelines)
6. [Dark/Light Mode](#darklight-mode)
7. [Implementation Guidelines](#implementation-guidelines)
8. [Common Mistakes to Avoid](#common-mistakes-to-avoid)

## Theme System Architecture

### Core Files

1. **Design Tokens**: `/client/src/lib/styles/tokens.css`
   - Defines all CSS custom properties (variables)
   - Core color palette, typography, spacing, shadows, transitions
   - Base values that themes build upon

2. **Theme Variants**: `/client/src/lib/styles/themes.css`
   - Light/dark theme definitions
   - High contrast themes
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

## Color System

### Design Philosophy

We follow ChatGPT.com's minimalist design approach:
- Clean, neutral color palette
- High contrast for readability
- Subtle UI elements that don't distract from content

### Color Tokens

**NEVER hardcode colors. Always use CSS variables through Tailwind classes or direct styles.**

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

### Current Theme Values

#### Dark Theme (Primary)
```css
--color-background-primary: #212121;    /* Very dark gray */
--color-background-secondary: #171717;  /* Darker for sidebars */
--color-background-tertiary: #0a0a0a;   /* Nearly black */
--color-text-primary: #e5e5e5;         /* Light gray */
--color-text-secondary: #a3a3a3;       /* Muted gray */
```

#### Light Theme
```css
--color-background-primary: #ffffff;    /* Pure white */
--color-background-secondary: #fafafa;  /* Off-white */
--color-background-tertiary: #f5f5f5;   /* Light gray */
--color-text-primary: #171717;         /* Nearly black */
--color-text-secondary: #525252;       /* Dark gray */
```

## Typography

### Font Families
- Sans: `font-sans` - System font stack for UI
- Mono: `font-mono` - Monospace for code

### Font Sizes
Use Tailwind's default scale: `text-xs`, `text-sm`, `text-base`, `text-lg`, etc.

### Font Weights
- Normal text: `font-normal` (400)
- Emphasized: `font-medium` (500)
- Headers: `font-semibold` (600) or `font-bold` (700)

## Spacing and Layout

### Spacing Scale
Use Tailwind's spacing scale consistently:
- `p-1` (0.25rem) to `p-12` (3rem) and beyond
- Common patterns: `p-4`, `px-6 py-4`, `gap-4`

### Container Widths
- Full width: `w-full`
- Constrained: `max-w-7xl mx-auto`
- Reading width: `max-w-prose`

### Responsive Design
- Mobile-first approach
- Breakpoints: `sm:`, `md:`, `lg:`, `xl:`, `2xl:`

## Component Guidelines

### Buttons
```html
<!-- Primary -->
<button class="bg-color-primary text-text-on-primary px-4 py-2 rounded-md hover:bg-color-primary-hover transition-colors">
  Action
</button>

<!-- Secondary -->
<button class="bg-bg-secondary text-text-primary border border-border-default px-4 py-2 rounded-md hover:bg-bg-tertiary transition-colors">
  Secondary
</button>
```

### Cards
```html
<div class="bg-bg-elevated border border-border-default rounded-lg p-6 shadow-sm">
  <!-- Content -->
</div>
```

### Forms
```html
<input
  class="w-full bg-bg-secondary text-text-primary border border-border-default rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-color-primary focus:border-transparent"
  type="text"
  placeholder="Enter text..."
/>
```

## Dark/Light Mode

### Implementation
1. Theme toggle updates the store: `$theme = 'light' | 'dark' | 'system'`
2. Store applies class to `<html>` element
3. CSS variables change based on `.light` or `.dark` class
4. All colors automatically update

### Best Practices
- Never use `dark:` prefixes in Tailwind
- Always use semantic color tokens that adapt to theme
- Test both themes during development

## Implementation Guidelines

### For New Components

1. **Always check existing components first** for patterns to follow
2. **Use only theme-aware classes**:
   ```html
   <!-- ❌ BAD -->
   <div class="bg-gray-800 text-white dark:bg-white dark:text-black">

   <!-- ✅ GOOD -->
   <div class="bg-bg-primary text-text-primary">
   ```

3. **Import global styles** in app.html:
   ```html
   <link rel="stylesheet" href="%sveltekit.assets%/styles/tokens.css" />
   <link rel="stylesheet" href="%sveltekit.assets%/styles/themes.css" />
   ```

4. **Use the theme store** for theme-aware logic:
   ```typescript
   import { theme } from '$lib/stores/theme';

   $: isDark = $theme === 'dark';
   ```

### For Existing Components

When updating existing components to use the theme system:

1. **Identify all hardcoded colors**
2. **Replace with semantic tokens**:
   - `bg-gray-50` → `bg-bg-primary`
   - `text-gray-900` → `text-text-primary`
   - `border-gray-200` → `border-border-default`
   - etc.

3. **Test in both themes**
4. **Check hover/focus states**

## Common Mistakes to Avoid

### ❌ DON'T

1. **Hardcode colors**
   ```html
   <div class="bg-gray-800 text-white">
   ```

2. **Use dark: prefixes**
   ```html
   <div class="bg-white dark:bg-gray-800">
   ```

3. **Mix semantic and literal colors**
   ```html
   <div class="bg-bg-primary text-white">
   ```

4. **Forget status states**
   ```html
   <div class="bg-red-500 text-white">
   ```

### ✅ DO

1. **Use semantic tokens**
   ```html
   <div class="bg-bg-primary text-text-primary">
   ```

2. **Let CSS variables handle themes**
   ```html
   <div class="bg-bg-primary"> <!-- Automatically adapts -->
   ```

3. **Use consistent semantic tokens**
   ```html
   <div class="bg-bg-primary text-text-primary">
   ```

4. **Use status color tokens**
   ```html
   <div class="bg-color-error-background text-color-error">
   ```

## Testing Checklist

Before committing any UI changes:

- [ ] Component works in light mode
- [ ] Component works in dark mode
- [ ] No hardcoded colors
- [ ] Follows spacing guidelines
- [ ] Responsive on mobile/tablet/desktop
- [ ] Accessible (proper contrast, focus states)
- [ ] Consistent with existing components

## Migration Guide

### Updating the OAuth Callback Page

The OAuth callback page currently uses hardcoded colors. Here's how to fix it:

```html
<!-- Current (incorrect) -->
<div class="min-h-screen bg-gray-50 flex items-center justify-center">
  <div class="bg-white p-8 rounded-lg shadow-md">
    <h1 class="text-2xl font-bold text-gray-900">Processing...</h1>
    <p class="text-gray-600">Redirecting you back to the application...</p>

    <!-- Error state -->
    <div class="bg-red-50 text-red-600 p-4 rounded">
      Error message
    </div>
  </div>
</div>

<!-- Updated (correct) -->
<div class="min-h-screen bg-bg-primary flex items-center justify-center">
  <div class="bg-bg-elevated p-8 rounded-lg shadow-md border border-border-default">
    <h1 class="text-2xl font-bold text-text-primary">Processing...</h1>
    <p class="text-text-secondary">Redirecting you back to the application...</p>

    <!-- Error state -->
    <div class="bg-color-error-background text-color-error p-4 rounded">
      Error message
    </div>
  </div>
</div>
```

## References

- [Tailwind Configuration](/client/tailwind.config.js)
- [Design Tokens](/client/src/lib/styles/tokens.css)
- [Theme Variants](/client/src/lib/styles/themes.css)
- [Theme Store](/client/src/lib/stores/theme.ts)
- [Architecture Documentation](./ARCHITECTURE.md#user-interface)
