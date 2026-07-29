import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import ObsidianPreviewPanel from '../../src/routes/(app)/preferences/integrations/obsidian/components/ObsidianPreviewPanel.svelte';

describe('ObsidianPreviewPanel', () => {
	it('renders note previews and rerender controls', async () => {
		const onRenderPreview = vi.fn();
		const onSetPreviewView = vi.fn();

		render(ObsidianPreviewPanel, {
			props: {
				previewView: 'note',
				previewing: false,
				previewFilePath: 'Indelible/articles/Sample.md',
				previewBody: '# Sample',
				previewBodyHtml: '<h1>Sample</h1>',
				previewError: null,
				fullTextOffAndMissing: false,
				fullTextMissing: false,
				previewMissingSummary: true,
				onEnableFullText: vi.fn(),
				onRenderPreview,
				onSetPreviewView
			}
		});

		expect(screen.getByText('vault path')).toBeTruthy();
		expect(screen.getByText('Indelible/articles/Sample.md')).toBeTruthy();
		expect(screen.getByText(/No summary rendered for this preview/)).toBeTruthy();

		await fireEvent.click(screen.getByRole('button', { name: /re-render/i }));
		expect(onRenderPreview).toHaveBeenCalledOnce();

		await fireEvent.click(screen.getByRole('tab', { name: /full text/i }));
		expect(onSetPreviewView).toHaveBeenCalledWith('full');
	});

	it('offers to enable full text when the preview requires it', async () => {
		const onEnableFullText = vi.fn();

		render(ObsidianPreviewPanel, {
			props: {
				previewView: 'full',
				previewing: false,
				previewFilePath: '',
				previewBody: '',
				previewBodyHtml: '',
				previewError: null,
				fullTextOffAndMissing: true,
				fullTextMissing: false,
				previewMissingSummary: false,
				onEnableFullText,
				onRenderPreview: vi.fn(),
				onSetPreviewView: vi.fn()
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: /turn on/i }));
		expect(onEnableFullText).toHaveBeenCalledOnce();
	});
});
