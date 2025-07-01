<!-- Markdown content renderer -->

<script lang="ts">
	/* eslint-disable svelte/no-at-html-tags */
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';

	export let content: string;
	export let className: string = '';

	let sanitizedHtml: string = '';

	// Configure marked options
	marked.setOptions({
		gfm: true,
		breaks: true,
		pedantic: false
	});

	$: {
		try {
			// Parse markdown
			const rawHtml = marked.parse(content) as string;
			// Sanitize HTML to prevent XSS
			sanitizedHtml = DOMPurify.sanitize(rawHtml, {
				ALLOWED_TAGS: [
					'h1',
					'h2',
					'h3',
					'h4',
					'h5',
					'h6',
					'p',
					'br',
					'hr',
					'strong',
					'em',
					'code',
					'pre',
					'blockquote',
					'q',
					'ul',
					'ol',
					'li',
					'a',
					'table',
					'thead',
					'tbody',
					'tr',
					'th',
					'td',
					'img',
					'div',
					'span'
				],
				ALLOWED_ATTR: ['href', 'title', 'target', 'rel', 'class', 'src', 'alt', 'width', 'height']
			});
		} catch (error) {
			console.error('Error parsing markdown:', error);
			sanitizedHtml = DOMPurify.sanitize(content);
		}
	}
</script>

<div
	class="markdown-content prose prose-sm dark:prose-invert prose-headings:text-text-primary prose-p:text-text-primary prose-strong:text-text-primary prose-a:text-action-primary prose-code:text-text-accent max-w-none {className}"
>
	{@html sanitizedHtml}
</div>

<style>
	:global(.markdown-content) {
		word-break: break-word;
		color: inherit;
	}

	:global(.markdown-content > *:first-child) {
		margin-top: 0 !important;
	}

	:global(.markdown-content > *:last-child) {
		margin-bottom: 0 !important;
	}

	/* Headings */
	:global(.markdown-content h1) {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
		margin-top: var(--space-4);
		margin-bottom: var(--space-2);
	}

	:global(.markdown-content h2) {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-bold);
		margin-top: var(--space-4);
		margin-bottom: var(--space-2);
	}

	:global(.markdown-content h3) {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		margin-top: var(--space-4);
		margin-bottom: var(--space-2);
	}

	:global(.markdown-content h4),
	:global(.markdown-content h5),
	:global(.markdown-content h6) {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		margin-top: var(--space-4);
		margin-bottom: var(--space-2);
	}

	/* Paragraphs and text */
	:global(.markdown-content p) {
		margin-bottom: var(--space-4);
		line-height: var(--line-height-relaxed);
	}

	:global(.markdown-content strong) {
		font-weight: var(--font-weight-semibold);
	}

	/* Code blocks */
	:global(.markdown-content pre) {
		margin: 1em 0;
		padding: 1rem;
		background-color: var(--color-surface-secondary);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: var(--radius-md);
		overflow-x: auto;
		font-size: var(--font-size-sm);
	}

	:global(.markdown-content code:not(pre code)) {
		padding: 0.125rem 0.25rem;
		background-color: var(--color-background-accent);
		color: var(--color-text-accent);
		border-radius: var(--radius-sm);
		font-size: var(--font-size-sm);
	}

	/* Lists */
	:global(.markdown-content ul) {
		list-style-type: disc;
		margin-left: 1.5rem;
		margin-bottom: 1em;
	}

	:global(.markdown-content ol) {
		list-style-type: decimal;
		margin-left: 1.5rem;
		margin-bottom: 1em;
	}

	:global(.markdown-content li) {
		margin-bottom: 0.25rem;
	}

	/* Blockquotes */
	:global(.markdown-content blockquote) {
		border-left: 4px solid var(--color-border-accent);
		padding-left: 1rem;
		font-style: italic;
		margin: 1em 0;
		color: var(--color-text-secondary);
	}

	/* Links */
	:global(.markdown-content a) {
		color: var(--color-action-primary);
		text-decoration: underline;
		transition: color var(--transition-duration-fast) var(--transition-easing-smooth);
	}

	:global(.markdown-content a:hover) {
		color: var(--color-action-primary-hover);
	}

	/* Tables */
	:global(.markdown-content table) {
		width: 100%;
		margin: 0.5rem 0;
		font-size: 0.875rem;
		border-collapse: collapse;
	}

	:global(.markdown-content th) {
		background-color: var(--color-surface-secondary);
		padding: 0.5rem 1rem;
		text-align: left;
		font-weight: var(--font-weight-medium);
		border-bottom: 1px solid var(--color-border-default);
	}

	:global(.markdown-content td) {
		padding: 0.5rem 1rem;
		border-bottom: 1px solid var(--color-border-light);
	}

	/* Horizontal rules */
	:global(.markdown-content hr) {
		margin: 1rem 0;
		border: 0;
		border-top: 1px solid var(--color-border-default);
	}

	/* Images */
	:global(.markdown-content img) {
		max-width: 100%;
		height: auto;
		border-radius: var(--radius-md);
	}
</style>
