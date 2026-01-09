import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Input from './input.svelte';

describe('Input', () => {
	describe('rendering', () => {
		it('should render an input element', () => {
			render(Input);
			expect(screen.getByRole('textbox')).toBeInTheDocument();
		});

		it('should have default type text', () => {
			render(Input);
			const input = screen.getByRole('textbox');
			expect(input).toHaveAttribute('type', 'text');
		});

		it('should apply base styling classes', () => {
			render(Input);
			const input = screen.getByRole('textbox');
			expect(input).toHaveClass('rounded-md');
			expect(input).toHaveClass('border');
			expect(input).toHaveClass('w-full');
		});
	});

	describe('label', () => {
		it('should render label when provided', () => {
			render(Input, { props: { label: 'Email' } });
			expect(screen.getByText('Email')).toBeInTheDocument();
		});

		it('should associate label with input', () => {
			render(Input, { props: { label: 'Email' } });
			const input = screen.getByRole('textbox');
			const label = screen.getByText('Email');
			expect(label).toHaveAttribute('for', input.id);
		});

		it('should not render label when not provided', () => {
			render(Input);
			expect(screen.queryByRole('label')).not.toBeInTheDocument();
		});
	});

	describe('value binding', () => {
		it('should accept initial value', () => {
			render(Input, { props: { value: 'initial' } });
			const input = screen.getByRole('textbox');
			expect(input).toHaveValue('initial');
		});

		it('should update value on input', async () => {
			render(Input);
			const input = screen.getByRole('textbox');

			await fireEvent.input(input, { target: { value: 'new value' } });

			expect(input).toHaveValue('new value');
		});
	});

	describe('input types', () => {
		it('should render email type', () => {
			render(Input, { props: { type: 'email' } });
			const input = screen.getByRole('textbox');
			expect(input).toHaveAttribute('type', 'email');
		});

		it('should render password type', () => {
			render(Input, { props: { type: 'password' } });
			// Password inputs don't have textbox role
			const input = document.querySelector('input[type="password"]');
			expect(input).toBeInTheDocument();
		});

		it('should render number type', () => {
			render(Input, { props: { type: 'number' } });
			const input = screen.getByRole('spinbutton');
			expect(input).toHaveAttribute('type', 'number');
		});
	});

	describe('error state', () => {
		it('should apply error styles when error is true', () => {
			render(Input, { props: { error: true } });
			const input = screen.getByRole('textbox');
			expect(input).toHaveClass('border-status-error');
			expect(input).toHaveClass('bg-status-error-bg');
		});

		it('should apply error styles when error is a string', () => {
			render(Input, { props: { error: 'Invalid email' } });
			const input = screen.getByRole('textbox');
			expect(input).toHaveClass('border-status-error');
		});

		it('should display error message when error is a string', () => {
			render(Input, { props: { error: 'Invalid email' } });
			expect(screen.getByText('Invalid email')).toBeInTheDocument();
		});

		it('should apply error text styling to error message', () => {
			render(Input, { props: { error: 'Invalid email' } });
			const errorText = screen.getByText('Invalid email');
			expect(errorText).toHaveClass('text-status-error');
		});

		it('should not apply error styles when error is false', () => {
			render(Input, { props: { error: false } });
			const input = screen.getByRole('textbox');
			expect(input).not.toHaveClass('border-status-error');
			expect(input).toHaveClass('border-border-default');
		});

		it('should not apply error styles when error is empty string', () => {
			render(Input, { props: { error: '' } });
			const input = screen.getByRole('textbox');
			expect(input).not.toHaveClass('border-status-error');
		});
	});

	describe('helper text', () => {
		it('should display helperText', () => {
			render(Input, { props: { helperText: 'Enter your email' } });
			expect(screen.getByText('Enter your email')).toBeInTheDocument();
		});

		it('should display helpText as fallback', () => {
			render(Input, { props: { helpText: 'Enter your email' } });
			expect(screen.getByText('Enter your email')).toBeInTheDocument();
		});

		it('should apply secondary text styling to helper text', () => {
			render(Input, { props: { helperText: 'Enter your email' } });
			const helperText = screen.getByText('Enter your email');
			expect(helperText).toHaveClass('text-text-secondary');
		});

		it('should prioritize error message over helper text', () => {
			render(Input, { props: { error: 'Invalid', helperText: 'Enter email' } });
			expect(screen.getByText('Invalid')).toBeInTheDocument();
			expect(screen.queryByText('Enter email')).not.toBeInTheDocument();
		});

		it('should have aria-live for accessibility', () => {
			render(Input, { props: { helperText: 'Enter your email' } });
			const helperText = screen.getByText('Enter your email');
			expect(helperText).toHaveAttribute('aria-live', 'polite');
		});
	});

	describe('placeholder', () => {
		it('should display placeholder text', () => {
			render(Input, { props: { placeholder: 'Enter value...' } });
			const input = screen.getByPlaceholderText('Enter value...');
			expect(input).toBeInTheDocument();
		});
	});

	describe('disabled state', () => {
		it('should be disabled when disabled prop is true', () => {
			render(Input, { props: { disabled: true } });
			const input = screen.getByRole('textbox');
			expect(input).toBeDisabled();
		});

		it('should have disabled styling', () => {
			render(Input, { props: { disabled: true } });
			const input = screen.getByRole('textbox');
			expect(input).toHaveClass('disabled:opacity-50');
			expect(input).toHaveClass('disabled:cursor-not-allowed');
		});
	});

	describe('event handlers', () => {
		it('should call onblur when input loses focus', async () => {
			const handleBlur = vi.fn();
			render(Input, { props: { onblur: handleBlur } });
			const input = screen.getByRole('textbox');

			await fireEvent.blur(input);

			expect(handleBlur).toHaveBeenCalledTimes(1);
		});
	});

	describe('custom id', () => {
		it('should use provided id', () => {
			render(Input, { props: { id: 'custom-id', label: 'Custom' } });
			const input = screen.getByRole('textbox');
			const label = screen.getByText('Custom');
			expect(input).toHaveAttribute('id', 'custom-id');
			expect(label).toHaveAttribute('for', 'custom-id');
		});

		it('should generate unique id when not provided', () => {
			render(Input);
			const input = screen.getByRole('textbox');
			expect(input.id).toMatch(/^input-[a-z0-9]+$/);
		});
	});

	describe('custom class', () => {
		it('should apply custom class alongside default classes', () => {
			render(Input, { props: { class: 'my-custom-class' } });
			const input = screen.getByRole('textbox');
			expect(input).toHaveClass('my-custom-class');
			expect(input).toHaveClass('rounded-md'); // base class still present
		});
	});

	describe('accessibility', () => {
		it('should have focus ring styling', () => {
			render(Input);
			const input = screen.getByRole('textbox');
			expect(input).toHaveClass('focus-visible:ring-2');
			expect(input).toHaveClass('focus-visible:ring-amber-400');
		});

		it('should have minimum touch target height', () => {
			render(Input);
			const input = screen.getByRole('textbox');
			expect(input).toHaveClass('min-h-touch-target');
		});

		it('should associate helper text with input via id', () => {
			render(Input, { props: { id: 'test-input', helperText: 'Helper' } });
			const helperText = screen.getByText('Helper');
			expect(helperText).toHaveAttribute('id', 'test-input-description');
		});
	});
});
