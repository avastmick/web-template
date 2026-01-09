import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Card from './card.svelte';

describe('Card', () => {
	describe('rendering', () => {
		it('should render a div element', () => {
			const { container } = render(Card);
			expect(container.querySelector('div')).toBeInTheDocument();
		});

		it('should apply default variant classes', () => {
			const { container } = render(Card);
			const card = container.firstElementChild;
			expect(card).toHaveClass('bg-surface-primary');
			expect(card).toHaveClass('border');
			expect(card).toHaveClass('border-border-default');
		});

		it('should apply default padding classes', () => {
			const { container } = render(Card);
			const card = container.firstElementChild;
			expect(card).toHaveClass('p-4');
		});

		it('should apply default rounded classes', () => {
			const { container } = render(Card);
			const card = container.firstElementChild;
			expect(card).toHaveClass('rounded-lg');
		});
	});

	describe('variants', () => {
		it('should apply default variant', () => {
			const { container } = render(Card, { props: { variant: 'default' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('bg-surface-primary');
			expect(card).toHaveClass('border-border-default');
		});

		it('should apply raised variant', () => {
			const { container } = render(Card, { props: { variant: 'raised' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('bg-surface-raised');
			expect(card).toHaveClass('shadow-md');
			expect(card).toHaveClass('hover:shadow-lg');
		});

		it('should apply outlined variant', () => {
			const { container } = render(Card, { props: { variant: 'outlined' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('bg-transparent');
			expect(card).toHaveClass('border-2');
			expect(card).toHaveClass('border-border-dark');
		});

		it('should apply ghost variant', () => {
			const { container } = render(Card, { props: { variant: 'ghost' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('bg-surface-secondary');
		});
	});

	describe('padding', () => {
		it('should apply no padding', () => {
			const { container } = render(Card, { props: { padding: 'none' } });
			const card = container.firstElementChild;
			expect(card).not.toHaveClass('p-3');
			expect(card).not.toHaveClass('p-4');
			expect(card).not.toHaveClass('p-6');
		});

		it('should apply small padding', () => {
			const { container } = render(Card, { props: { padding: 'sm' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('p-3');
		});

		it('should apply medium padding', () => {
			const { container } = render(Card, { props: { padding: 'md' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('p-4');
		});

		it('should apply large padding', () => {
			const { container } = render(Card, { props: { padding: 'lg' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('p-6');
		});
	});

	describe('rounded', () => {
		it('should apply no rounding', () => {
			const { container } = render(Card, { props: { rounded: 'none' } });
			const card = container.firstElementChild;
			expect(card).not.toHaveClass('rounded');
			expect(card).not.toHaveClass('rounded-md');
			expect(card).not.toHaveClass('rounded-lg');
			expect(card).not.toHaveClass('rounded-xl');
		});

		it('should apply small rounding', () => {
			const { container } = render(Card, { props: { rounded: 'sm' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('rounded');
		});

		it('should apply medium rounding', () => {
			const { container } = render(Card, { props: { rounded: 'md' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('rounded-md');
		});

		it('should apply large rounding', () => {
			const { container } = render(Card, { props: { rounded: 'lg' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('rounded-lg');
		});

		it('should apply xl rounding', () => {
			const { container } = render(Card, { props: { rounded: 'xl' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('rounded-xl');
		});
	});

	describe('custom class', () => {
		it('should apply custom class alongside default classes', () => {
			const { container } = render(Card, { props: { class: 'my-custom-class' } });
			const card = container.firstElementChild;
			expect(card).toHaveClass('my-custom-class');
			expect(card).toHaveClass('rounded-lg'); // default rounding still present
		});
	});

	describe('combination of props', () => {
		it('should apply multiple props together', () => {
			const { container } = render(Card, {
				props: {
					variant: 'raised',
					padding: 'lg',
					rounded: 'xl'
				}
			});
			const card = container.firstElementChild;
			expect(card).toHaveClass('bg-surface-raised');
			expect(card).toHaveClass('shadow-md');
			expect(card).toHaveClass('p-6');
			expect(card).toHaveClass('rounded-xl');
		});
	});

	describe('rest props', () => {
		it('should spread additional props to div', () => {
			const { container } = render(Card, { props: { 'data-testid': 'custom-card' } });
			const card = container.firstElementChild;
			expect(card).toHaveAttribute('data-testid', 'custom-card');
		});
	});
});
