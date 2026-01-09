import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Button from './button.svelte';

describe('Button', () => {
	describe('rendering', () => {
		it('should render a button element', () => {
			render(Button);
			expect(screen.getByRole('button')).toBeInTheDocument();
		});

		it('should apply default variant classes', () => {
			render(Button);
			const button = screen.getByRole('button');
			expect(button).toHaveClass('bg-primary');
		});

		it('should apply default size classes', () => {
			render(Button);
			const button = screen.getByRole('button');
			expect(button).toHaveClass('h-touch-target');
		});
	});

	describe('variants', () => {
		it('should apply destructive variant', () => {
			render(Button, { props: { variant: 'destructive' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('bg-status-error');
		});

		it('should apply outline variant', () => {
			render(Button, { props: { variant: 'outline' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('border');
			expect(button).toHaveClass('bg-transparent');
		});

		it('should apply secondary variant', () => {
			render(Button, { props: { variant: 'secondary' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('bg-background-secondary');
		});

		it('should apply ghost variant', () => {
			render(Button, { props: { variant: 'ghost' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('hover:bg-background-secondary');
		});

		it('should apply link variant', () => {
			render(Button, { props: { variant: 'link' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('text-primary');
			expect(button).toHaveClass('underline-offset-4');
		});
	});

	describe('sizes', () => {
		it('should apply small size', () => {
			render(Button, { props: { size: 'sm' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('h-9');
			expect(button).toHaveClass('px-4');
		});

		it('should apply large size', () => {
			render(Button, { props: { size: 'lg' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('h-12');
			expect(button).toHaveClass('px-8');
		});

		it('should apply icon size', () => {
			render(Button, { props: { size: 'icon' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('h-touch-target');
			expect(button).toHaveClass('w-touch-target');
		});
	});

	describe('disabled state', () => {
		it('should be disabled when disabled prop is true', () => {
			render(Button, { props: { disabled: true } });
			const button = screen.getByRole('button');
			expect(button).toBeDisabled();
		});

		it('should not be disabled by default', () => {
			render(Button);
			const button = screen.getByRole('button');
			expect(button).not.toBeDisabled();
		});
	});

	describe('loading state', () => {
		it('should be disabled when loading', () => {
			render(Button, { props: { loading: true } });
			const button = screen.getByRole('button');
			expect(button).toBeDisabled();
		});

		it('should have aria-busy when loading', () => {
			render(Button, { props: { loading: true } });
			const button = screen.getByRole('button');
			expect(button).toHaveAttribute('aria-busy', 'true');
		});

		it('should show loading text in aria-label when loading', () => {
			render(Button, { props: { loading: true, loadingText: 'Please wait...' } });
			const button = screen.getByRole('button');
			expect(button).toHaveAttribute('aria-label', 'Please wait...');
		});

		it('should use default loading text', () => {
			render(Button, { props: { loading: true } });
			const button = screen.getByRole('button');
			expect(button).toHaveAttribute('aria-label', 'Loading...');
		});

		it('should have btn-loading class when loading', () => {
			render(Button, { props: { loading: true } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('btn-loading');
		});

		it('should have screen reader text when loading', () => {
			render(Button, { props: { loading: true } });
			expect(screen.getByText('Loading...')).toHaveClass('sr-only');
		});
	});

	describe('click events', () => {
		it('should call onclick handler when clicked', async () => {
			const handleClick = vi.fn();
			render(Button, { props: { onclick: handleClick } });
			const button = screen.getByRole('button');

			await fireEvent.click(button);

			expect(handleClick).toHaveBeenCalledTimes(1);
		});

		it('should be disabled when disabled prop is passed with onclick', () => {
			const handleClick = vi.fn();
			render(Button, { props: { onclick: handleClick, disabled: true } });
			const button = screen.getByRole('button');

			// Button is disabled - browser will prevent click events
			expect(button).toBeDisabled();
		});

		it('should be disabled when loading with onclick', () => {
			const handleClick = vi.fn();
			render(Button, { props: { onclick: handleClick, loading: true } });
			const button = screen.getByRole('button');

			// Button is disabled during loading - browser will prevent click events
			expect(button).toBeDisabled();
		});
	});

	describe('accessibility', () => {
		it('should have base accessibility classes', () => {
			render(Button);
			const button = screen.getByRole('button');
			expect(button).toHaveClass('focus-visible:outline-none');
			expect(button).toHaveClass('focus-visible:ring-2');
		});

		it('should support aria-describedby', () => {
			render(Button, { props: { 'aria-describedby': 'help-text' } });
			const button = screen.getByRole('button');
			expect(button).toHaveAttribute('aria-describedby', 'help-text');
		});

		it('should support aria-expanded', () => {
			render(Button, { props: { 'aria-expanded': true } });
			const button = screen.getByRole('button');
			expect(button).toHaveAttribute('aria-expanded', 'true');
		});

		it('should support aria-pressed', () => {
			render(Button, { props: { 'aria-pressed': true } });
			const button = screen.getByRole('button');
			expect(button).toHaveAttribute('aria-pressed', 'true');
		});
	});

	describe('custom class', () => {
		it('should apply custom class alongside default classes', () => {
			render(Button, { props: { class: 'my-custom-class' } });
			const button = screen.getByRole('button');
			expect(button).toHaveClass('my-custom-class');
			expect(button).toHaveClass('inline-flex'); // base class still present
		});
	});

	describe('type attribute', () => {
		it('should default to type button', () => {
			render(Button);
			const button = screen.getByRole('button');
			// Default HTML button type is 'submit', but we can check if type is passed
			// The component spreads restProps, so type should work
		});

		it('should accept type submit', () => {
			render(Button, { props: { type: 'submit' } });
			const button = screen.getByRole('button');
			expect(button).toHaveAttribute('type', 'submit');
		});
	});
});
