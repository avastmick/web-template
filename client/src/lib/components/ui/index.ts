/**
 * UI Components - Modern, Accessible Component Library
 *
 * Built on shadcn-svelte patterns with our design system
 * - All components follow WCAG 2.1 AA accessibility standards
 * - Minimum 44px touch targets for mobile accessibility
 * - Consistent design tokens and theming support
 * - TypeScript support with proper type exports
 */

// Base components
export { default as Button } from './button.svelte';
export { default as Input } from './input.svelte';

// Layout components
export { default as Container } from './container.svelte';
export { default as Grid } from './grid.svelte';
export { default as Flex } from './flex.svelte';

// Feedback components
export { default as Alert } from './alert.svelte';
export { default as Card } from './card.svelte';

// Form components
export { default as FormField } from './form-field.svelte';

// Re-export component types for TypeScript users
export type { ComponentProps } from 'svelte';
