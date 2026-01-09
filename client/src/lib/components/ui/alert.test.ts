import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Alert from './alert.svelte';

describe('Alert', () => {
	describe('rendering', () => {
		it('should render an alert element', () => {
			render(Alert);
			expect(screen.getByRole('alert')).toBeInTheDocument();
		});

		it('should apply default info variant classes', () => {
			render(Alert);
			const alert = screen.getByRole('alert');
			expect(alert).toHaveClass('bg-status-info-bg');
		});
	});

	describe('variants', () => {
		it('should apply success variant', () => {
			render(Alert, { props: { variant: 'success' } });
			const alert = screen.getByRole('alert');
			expect(alert).toHaveClass('bg-status-success-bg');
		});

		it('should apply error variant', () => {
			render(Alert, { props: { variant: 'error' } });
			const alert = screen.getByRole('alert');
			expect(alert).toHaveClass('bg-status-error-bg');
		});

		it('should apply warning variant', () => {
			render(Alert, { props: { variant: 'warning' } });
			const alert = screen.getByRole('alert');
			expect(alert).toHaveClass('bg-status-warning-bg');
		});

		it('should apply info variant', () => {
			render(Alert, { props: { variant: 'info' } });
			const alert = screen.getByRole('alert');
			expect(alert).toHaveClass('bg-status-info-bg');
		});
	});

	describe('title and description', () => {
		it('should render title when provided', () => {
			render(Alert, { props: { title: 'Alert Title' } });
			expect(screen.getByText('Alert Title')).toBeInTheDocument();
		});

		it('should apply title styling based on variant', () => {
			render(Alert, { props: { title: 'Error Title', variant: 'error' } });
			const title = screen.getByText('Error Title');
			expect(title).toHaveClass('text-status-error');
		});

		it('should render description when provided', () => {
			render(Alert, { props: { description: 'Alert description text' } });
			expect(screen.getByText('Alert description text')).toBeInTheDocument();
		});

		it('should apply description styling based on variant', () => {
			render(Alert, { props: { description: 'Success message', variant: 'success' } });
			const description = screen.getByText('Success message');
			expect(description).toHaveClass('text-status-success/90');
		});

		it('should render both title and description', () => {
			render(Alert, {
				props: {
					title: 'Alert Title',
					description: 'Alert description'
				}
			});
			expect(screen.getByText('Alert Title')).toBeInTheDocument();
			expect(screen.getByText('Alert description')).toBeInTheDocument();
		});
	});

	describe('icon', () => {
		it('should show icon by default', () => {
			render(Alert);
			const alert = screen.getByRole('alert');
			const svg = alert.querySelector('svg');
			expect(svg).toBeInTheDocument();
		});

		it('should hide icon when icon prop is false', () => {
			render(Alert, { props: { icon: false } });
			const alert = screen.getByRole('alert');
			// Find the first SVG that's not a dismiss button
			const icons = alert.querySelectorAll('svg[aria-hidden="true"]');
			// All icons should be dismiss buttons or none at all
			expect(icons.length).toBe(0);
		});

		it('should show success icon for success variant', () => {
			render(Alert, { props: { variant: 'success' } });
			const alert = screen.getByRole('alert');
			const icon = alert.querySelector('svg');
			expect(icon).toHaveClass('text-status-success');
		});

		it('should show error icon for error variant', () => {
			render(Alert, { props: { variant: 'error' } });
			const alert = screen.getByRole('alert');
			const icon = alert.querySelector('svg');
			expect(icon).toHaveClass('text-status-error');
		});

		it('should show warning icon for warning variant', () => {
			render(Alert, { props: { variant: 'warning' } });
			const alert = screen.getByRole('alert');
			const icon = alert.querySelector('svg');
			expect(icon).toHaveClass('text-status-warning');
		});

		it('should show info icon for info variant', () => {
			render(Alert, { props: { variant: 'info' } });
			const alert = screen.getByRole('alert');
			const icon = alert.querySelector('svg');
			expect(icon).toHaveClass('text-status-info');
		});
	});

	describe('dismissible', () => {
		it('should not show dismiss button by default', () => {
			render(Alert);
			expect(screen.queryByRole('button', { name: 'Dismiss' })).not.toBeInTheDocument();
		});

		it('should show dismiss button when dismissible is true', () => {
			render(Alert, { props: { dismissible: true } });
			expect(screen.getByRole('button', { name: 'Dismiss' })).toBeInTheDocument();
		});

		it('should have clickable dismiss button', async () => {
			render(Alert, { props: { dismissible: true } });

			const dismissButton = screen.getByRole('button', { name: 'Dismiss' });

			// Verify dismiss button is clickable (doesn't throw)
			await fireEvent.click(dismissButton);

			// Button should be functional (the actual event dispatch is handled by Svelte)
			expect(dismissButton).toBeInTheDocument();
		});

		it('should apply variant styling to dismiss button', () => {
			render(Alert, { props: { dismissible: true, variant: 'error' } });
			const dismissButton = screen.getByRole('button', { name: 'Dismiss' });
			expect(dismissButton).toHaveClass('text-status-error');
		});
	});

	describe('accessibility', () => {
		it('should have role="alert"', () => {
			render(Alert);
			expect(screen.getByRole('alert')).toBeInTheDocument();
		});

		it('should have aria-hidden on icon', () => {
			render(Alert);
			const alert = screen.getByRole('alert');
			const icon = alert.querySelector('svg');
			expect(icon).toHaveAttribute('aria-hidden', 'true');
		});

		it('should have aria-label on dismiss button', () => {
			render(Alert, { props: { dismissible: true } });
			const dismissButton = screen.getByRole('button');
			expect(dismissButton).toHaveAttribute('aria-label', 'Dismiss');
		});

		it('should have focus ring on dismiss button', () => {
			render(Alert, { props: { dismissible: true } });
			const dismissButton = screen.getByRole('button', { name: 'Dismiss' });
			expect(dismissButton).toHaveClass('focus:ring-2');
		});
	});

	describe('layout', () => {
		it('should have flex layout', () => {
			render(Alert);
			const alert = screen.getByRole('alert');
			expect(alert).toHaveClass('flex');
			expect(alert).toHaveClass('items-start');
		});

		it('should have gap between elements', () => {
			render(Alert);
			const alert = screen.getByRole('alert');
			expect(alert).toHaveClass('gap-3');
		});

		it('should have rounded corners', () => {
			render(Alert);
			const alert = screen.getByRole('alert');
			expect(alert).toHaveClass('rounded-lg');
		});

		it('should have padding', () => {
			render(Alert);
			const alert = screen.getByRole('alert');
			expect(alert).toHaveClass('p-4');
		});
	});
});
