import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ImportUploadCard from '$lib/components/imports/ImportUploadCard.svelte';

function makeFile(name: string, size: number, type: string): File {
	const file = new File([new Uint8Array(size)], name, { type });
	return file;
}

describe('ImportUploadCard', () => {
	it('renders the title and description', () => {
		render(ImportUploadCard, {
			props: {
				title: 'Readwise',
				description: 'Import your articles',
				onSubmit: () => {}
			}
		});
		expect(screen.getByText('Readwise')).toBeTruthy();
		expect(screen.getByText('Import your articles')).toBeTruthy();
		expect(screen.getByText(/Drop a file here/i)).toBeTruthy();
	});

	it('shows the file name after a valid file is selected', async () => {
		const onSubmit = vi.fn();
		render(ImportUploadCard, {
			props: {
				title: 'Readwise',
				acceptedExtensions: ['.html'],
				acceptedMimeTypes: ['text/html'],
				onSubmit
			}
		});
		const input = screen.getByTestId('file-input') as HTMLInputElement;
		const file = makeFile('export.html', 100, 'text/html');
		await fireEvent.change(input, { target: { files: [file] } });
		expect(screen.getByText('export.html')).toBeTruthy();
	});

	it('rejects files whose type and extension are unsupported', async () => {
		render(ImportUploadCard, {
			props: {
				title: 'Readwise',
				acceptedExtensions: ['.html'],
				acceptedMimeTypes: ['text/html'],
				onSubmit: () => {}
			}
		});
		const input = screen.getByTestId('file-input') as HTMLInputElement;
		const file = makeFile('data.json', 100, 'application/json');
		await fireEvent.change(input, { target: { files: [file] } });
		expect(screen.getByRole('alert').textContent).toMatch(/Unsupported file type/i);
		expect(screen.queryByText('data.json')).toBeNull();
	});

	it('rejects files larger than maxBytes', async () => {
		render(ImportUploadCard, {
			props: {
				title: 'Readwise',
				acceptedExtensions: ['.html'],
				maxBytes: 1024,
				onSubmit: () => {}
			}
		});
		const input = screen.getByTestId('file-input') as HTMLInputElement;
		const file = makeFile('export.html', 2048, 'text/html');
		await fireEvent.change(input, { target: { files: [file] } });
		expect(screen.getByRole('alert').textContent).toMatch(/too large/i);
	});

	it('calls onSubmit with the selected file when Start import is clicked', async () => {
		const onSubmit = vi.fn();
		render(ImportUploadCard, {
			props: {
				title: 'Readwise',
				acceptedExtensions: ['.html'],
				onSubmit
			}
		});
		const input = screen.getByTestId('file-input') as HTMLInputElement;
		const file = makeFile('export.html', 100, 'text/html');
		await fireEvent.change(input, { target: { files: [file] } });

		const button = screen.getByText('Start import').closest('button');
		expect(button).toBeTruthy();
		await fireEvent.click(button!);
		expect(onSubmit).toHaveBeenCalledTimes(1);
		expect(onSubmit.mock.calls[0]![0]?.name).toBe('export.html');
	});

	it('disables Start import while busy', async () => {
		render(ImportUploadCard, {
			props: {
				title: 'Readwise',
				acceptedExtensions: ['.html'],
				busy: true,
				onSubmit: () => {}
			}
		});
		const input = screen.getByTestId('file-input') as HTMLInputElement;
		const file = makeFile('export.html', 100, 'text/html');
		await fireEvent.change(input, { target: { files: [file] } });
		const button = screen.getByText('Start import').closest('button');
		expect(button?.hasAttribute('disabled')).toBe(true);
	});

	it('renders the externally-supplied error message', () => {
		render(ImportUploadCard, {
			props: {
				title: 'Readwise',
				errorMessage: 'upload failed',
				onSubmit: () => {}
			}
		});
		expect(screen.getByText('upload failed')).toBeTruthy();
	});

	it('uses "Start import" as the default submit label', () => {
		render(ImportUploadCard, {
			props: { title: 'Test', onSubmit: () => {} }
		});
		expect(screen.getByRole('button', { name: 'Start import' })).toBeTruthy();
	});

	it('renders a custom submitLabel when provided', () => {
		render(ImportUploadCard, {
			props: { title: 'Test', submitLabel: 'Upload Readwise CSV', onSubmit: () => {} }
		});
		expect(screen.getByRole('button', { name: 'Upload Readwise CSV' })).toBeTruthy();
	});

	it('clears the selection when Clear is clicked', async () => {
		render(ImportUploadCard, {
			props: {
				title: 'Readwise',
				acceptedExtensions: ['.html'],
				onSubmit: () => {}
			}
		});
		const input = screen.getByTestId('file-input') as HTMLInputElement;
		const file = makeFile('export.html', 100, 'text/html');
		await fireEvent.change(input, { target: { files: [file] } });
		const clear = screen.getByText('Clear').closest('button');
		await fireEvent.click(clear!);
		expect(screen.queryByText('export.html')).toBeNull();
	});
});
