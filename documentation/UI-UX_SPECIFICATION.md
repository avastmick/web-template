# Comprehensive UI/UX Specification for Modern Svelte SPA Template

## Minimalist UI design principles for 2024-2025

The current design philosophy emphasizes "Form follows feeling" - moving beyond pure simplicity to create interfaces that balance aesthetic minimalism with emotional resonance and functionality.

**Core Design System**:
- **Typography Scale**: Use maximum 4 font sizes (32-48px headers, 24-32px subheaders, 16-18px body, 14px small text)
- **Color System**: Follow 60-30-10 rule with neutral dominance, single accent color
- **Spacing Grid**: 8px base unit system (4px, 8px, 16px, 24px, 32px, 48px)
- **Micro-interactions**: 200-500ms duration with cubic-bezier(0.25, 0.46, 0.45, 0.94) easing

**Implementation Guidelines**:
```css
:root {
  /* Typography */
  --font-size-base: 1rem;
  --font-size-lg: 1.125rem;
  --font-size-xl: 1.5rem;
  --font-size-2xl: 2rem;

  /* Colors */
  --color-background: #FFFFFF;
  --color-surface: #F8F9FA;
  --color-text-primary: #212529;
  --color-text-secondary: #6C757D;
  --color-accent: #007BFF;

  /* Spacing (8px grid) */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-4: 1rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
}
```

## Svelte UI component library recommendations

**Primary Recommendation: shadcn-svelte**
- Zero bundle impact (copy-paste components)
- Built on Bits UI (headless) with Tailwind CSS styling
- Full TypeScript support and Tailwind CSS 4.0 compatibility
- Excellent accessibility out of the box

**Alternative Options**:
1. **Bits UI**: Best for custom design systems needing full control
2. **Flowbite Svelte**: Rapid prototyping with 59+ pre-built components
3. **Skeleton UI**: SvelteKit-specific with adaptive theming

**Implementation Pattern**:
```bash
npx shadcn-svelte@next init
npx shadcn-svelte@next add button dialog
```

## OAuth authentication UI patterns

**Universal OAuth Button Structure**:
```html
<button class="oauth-button oauth-button--[provider]"
        data-provider="[provider]"
        aria-label="Sign in with [Provider]">
  <img src="[provider-icon]" alt="[Provider] logo" class="oauth-button__icon">
  <span class="oauth-button__text">Continue with [Provider]</span>
</button>
```

**Design Specifications**:
- Minimum 44px height for touch targets
- Progressive disclosure flow (email → provider detection → authentication)
- Error handling with clear recovery paths
- Mobile-first responsive design

## Internationalization (i18n) implementation

**Recommended Approach**: svelte-i18n with lazy loading

**Key Patterns**:
```javascript
// Hierarchical key structure
{
  "page": {
    "home": {
      "title": "Homepage",
      "nav": "Home"
    }
  },
  "common": {
    "buttons": {
      "save": "Save",
      "cancel": "Cancel"
    }
  }
}

// RTL support
export const dir = derived(
  locale,
  $locale => ['ar', 'he', 'fa'].includes($locale) ? 'rtl' : 'ltr'
);
```

**Performance Optimization**:
- Route-based lazy loading of translations
- Namespace organization for efficient splitting
- Svelte store integration for reactive translations

## Theme switching with Tailwind CSS 4.0

**Implementation Strategy**:
```css
@import "tailwindcss";

@custom-variant dark (&:where(.dark, .dark *));

@theme {
  --color-primary: oklch(0.84 0.18 117.33);
  --color-surface: light-dark(#ffffff, #1a1a1a);
}
```

**Preventing Flash of Incorrect Theme**:
```html
<!-- In app.html -->
<script>
  const theme = localStorage.getItem('theme');
  const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  const isDark = theme === 'dark' || (!theme && systemDark);

  document.documentElement.classList.toggle('dark', isDark);
  document.documentElement.style.colorScheme = isDark ? 'dark' : 'light';
</script>
```

**Theme Store Pattern**:
```javascript
export const theme = writable('system');
export const resolvedTheme = derived([theme], ([$theme]) => {
  if ($theme === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return $theme;
});
```

## Performance optimization for SPAs without SSR

**Critical Strategies**:
1. **Code Splitting**: Dynamic imports for routes and heavy components
2. **Bundle Optimization**: Target <200KB initial JavaScript bundle
3. **Image Optimization**: Use WebP/AVIF with lazy loading
4. **Service Worker**: Implement caching for offline functionality

**Implementation Examples**:
```javascript
// Route-based code splitting
const routes = {
  '/': () => import('./routes/Home.svelte'),
  '/about': () => import('./routes/About.svelte'),
  '/dashboard': () => import('./routes/Dashboard.svelte')
};

// Lazy component loading
onMount(async () => {
  const module = await import('./HeavyComponent.svelte');
  component = module.default;
});
```

**Memory Leak Prevention**:
```javascript
import { onDestroy } from 'svelte';

let cleanup = [];

onDestroy(() => {
  cleanup.forEach(fn => fn());
  cleanup = [];
});
```

## Responsive design patterns

**Modern Layout Structure**:
```css
.app-container {
  display: grid;
  grid-template-areas:
    "header header"
    "sidebar main";
  grid-template-rows: 60px 1fr;
  grid-template-columns: 250px 1fr;
  min-height: 100dvh;
}

@media (max-width: 768px) {
  .app-container {
    grid-template-areas:
      "header"
      "main";
    grid-template-columns: 1fr;
  }
}
```

**Viewport Units Best Practices**:
- Use `dvh` for dynamic full-screen layouts
- Apply `svh` for consistent layouts with browser UI
- Implement safe area handling for iOS devices

**Touch-Friendly Guidelines**:
- Minimum 44×44px touch targets
- 8px minimum spacing between interactive elements
- Implement swipe gestures for mobile navigation

## Accessibility and WCAG compliance

**Core Requirements**:
- **Color Contrast**: 4.5:1 for normal text, 3:1 for large text
- **Keyboard Navigation**: All functionality accessible via keyboard
- **Screen Reader Support**: Proper semantic HTML and ARIA patterns
- **Focus Management**: Clear focus indicators and logical tab order

**Svelte-Specific Patterns**:
```svelte
<script>
  let expanded = false;
  let buttonId = 'dropdown-' + Math.random().toString(36);
</script>

<button
  id={buttonId}
  aria-expanded={expanded}
  aria-haspopup="true"
  on:click={() => expanded = !expanded}
>
  Menu
</button>

{#if expanded}
<ul role="menu" aria-labelledby={buttonId}>
  <li role="menuitem">Option 1</li>
  <li role="menuitem">Option 2</li>
</ul>
{/if}
```

**Testing Strategy**:
- Integrate axe-core into CI/CD pipeline
- Manual testing with NVDA/VoiceOver
- Regular accessibility audits

## AI-followable design token system

**W3C DTCG Standard Implementation**:
```json
{
  "$schema": "https://design-tokens.org/schema.json",
  "color": {
    "core": {
      "blue": {
        "500": {
          "$value": "#2563eb",
          "$type": "color",
          "$description": "Primary brand blue"
        }
      }
    },
    "semantic": {
      "action": {
        "primary": {
          "$value": "{color.core.blue.500}",
          "$type": "color",
          "$description": "Primary action color"
        }
      }
    }
  }
}
```

**Token Organization**:
- **Primitive Tokens**: Basic design values
- **Semantic Tokens**: Context-specific meanings
- **Component Tokens**: Component-specific applications

**Build Process with Style Dictionary**:
```javascript
module.exports = {
  platforms: {
    css: {
      transformGroup: 'css',
      buildPath: 'build/css/',
      files: [{
        destination: 'tokens.css',
        format: 'css/variables'
      }]
    },
    json: {
      buildPath: 'build/json/',
      files: [{
        destination: 'tokens.json',
        format: 'json/nested'
      }]
    }
  }
};
```

## Complete implementation architecture

**Project Structure**:
```
src/
├── lib/
│   ├── components/
│   │   ├── ui/           # UI components (buttons, forms, etc.)
│   │   └── layout/       # Layout components
│   ├── stores/
│   │   ├── theme.js      # Theme management
│   │   └── i18n.js       # Internationalization
│   ├── styles/
│   │   ├── tokens.css    # Design tokens
│   │   └── app.css       # Global styles
│   └── utils/
│       └── accessibility.js
├── routes/
└── app.html
```

**Technology Stack**:
- **Frontend**: Svelte 5 + Tailwind CSS 4.0
- **UI Components**: shadcn-svelte or Bits UI
- **State Management**: Svelte stores
- **i18n**: svelte-i18n with lazy loading
- **Build Tool**: Vite with Rollup optimizations
- **Testing**: Vitest + Playwright + axe-core

This specification provides a comprehensive foundation for building a modern, performant, and accessible Svelte SPA that follows current best practices and is structured for easy implementation by AI developer agents.
