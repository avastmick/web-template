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

<div class="markdown-content prose prose-sm dark:prose-invert max-w-none {className}">
	{@html sanitizedHtml}
</div>

<style>
	:global(.markdown-content) {
		word-break: break-word;
	}

	:global(.markdown-content > *:first-child) {
		margin-top: 0 !important;
	}

	:global(.markdown-content > *:last-child) {
		margin-bottom: 0 !important;
	}

	/* Headings */
	:global(.markdown-content h1) {
		font-size: 1.5rem;
		font-weight: 700;
		margin-top: 1rem;
		margin-bottom: 0.5rem;
	}

	:global(.markdown-content h2) {
		font-size: 1.25rem;
		font-weight: 700;
		margin-top: 1rem;
		margin-bottom: 0.5rem;
	}

	:global(.markdown-content h3) {
		font-size: 1.125rem;
		font-weight: 600;
		margin-top: 1rem;
		margin-bottom: 0.5rem;
	}

	:global(.markdown-content h4),
	:global(.markdown-content h5),
	:global(.markdown-content h6) {
		font-size: 1rem;
		font-weight: 600;
		margin-top: 1rem;
		margin-bottom: 0.5rem;
	}

	/* Paragraphs and text */
	:global(.markdown-content p) {
		margin-bottom: 1em;
		line-height: 1.625;
	}

	:global(.markdown-content strong) {
		font-weight: 600;
	}

	/* Code blocks */
	:global(.markdown-content pre) {
		margin: 1em 0;
		padding: 1rem;
		background-color: rgb(17 24 39);
		color: rgb(243 244 246);
		border-radius: 0.375rem;
		overflow-x: auto;
		font-size: 0.875rem;
	}

	:global(.markdown-content code:not(pre code)) {
		padding: 0.125rem 0.25rem;
		background-color: rgb(243 244 246);
		color: rgb(31 41 55);
		border-radius: 0.25rem;
		font-size: 0.875rem;
	}

	:global(.dark .markdown-content code:not(pre code)) {
		background-color: rgb(55 65 81);
		color: rgb(243 244 246);
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
		border-left: 4px solid rgb(209 213 219);
		padding-left: 1rem;
		font-style: italic;
		margin: 1em 0;
	}

	:global(.dark .markdown-content blockquote) {
		border-left-color: rgb(75 85 99);
		color: rgb(209 213 219);
	}

	/* Links */
	:global(.markdown-content a) {
		color: rgb(37 99 235);
		text-decoration: underline;
	}

	:global(.markdown-content a:hover) {
		color: rgb(29 78 216);
	}

	:global(.dark .markdown-content a) {
		color: rgb(96 165 250);
	}

	:global(.dark .markdown-content a:hover) {
		color: rgb(147 197 253);
	}

	/* Tables */
	:global(.markdown-content table) {
		width: 100%;
		margin: 0.5rem 0;
		font-size: 0.875rem;
		border-collapse: collapse;
	}

	:global(.markdown-content th) {
		background-color: rgb(249 250 251);
		padding: 0.5rem 1rem;
		text-align: left;
		font-weight: 500;
		border-bottom: 1px solid rgb(229 231 235);
	}

	:global(.dark .markdown-content th) {
		background-color: rgb(31 41 55);
		border-bottom-color: rgb(55 65 81);
	}

	:global(.markdown-content td) {
		padding: 0.5rem 1rem;
		border-bottom: 1px solid rgb(229 231 235);
	}

	:global(.dark .markdown-content td) {
		border-bottom-color: rgb(55 65 81);
	}

	/* Horizontal rules */
	:global(.markdown-content hr) {
		margin: 1rem 0;
		border: 0;
		border-top: 1px solid rgb(229 231 235);
	}

	:global(.dark .markdown-content hr) {
		border-top-color: rgb(55 65 81);
	}

	/* Images */
	:global(.markdown-content img) {
		max-width: 100%;
		height: auto;
		border-radius: 0.375rem;
	}

	/* Dark mode text */
	:global(.dark .markdown-content) {
		color: rgb(243 244 246);
	}
</style>
