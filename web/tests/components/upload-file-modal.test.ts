import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { tick } from 'svelte';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';

import UploadFileModal from '$lib/components/library/UploadFileModal.svelte';
import { getMaxUploadBytes } from '$lib/api/upload-limits';
import { getModalStore } from '$lib/stores/addItemModal.svelte';

vi.mock('$lib/api/upload-limits', () => ({
	getMaxUploadBytes: vi.fn(),
	resetUploadLimitsCache: vi.fn()
}));

const getMaxUploadBytesMock = vi.mocked(getMaxUploadBytes);

const MIB = 1024 * 1024;

/** Validation only reads `file.size`, so never allocate the declared bytes. */
function fileOfSize(name: string, size: number, type = 'application/pdf'): File {
	const file = new File(['x'], name, { type });
	Object.defineProperty(file, 'size', { value: size });
	return file;
}

async function openModal(): Promise<void> {
	getModalStore().open('upload');
	render(UploadFileModal);
	await waitFor(() => expect(getMaxUploadBytesMock).toHaveBeenCalled());
	await tick();
	await tick();
}

async function selectFile(file: File): Promise<void> {
	const input = screen.getByTestId('file-input');
	await fireEvent.change(input, { target: { files: [file] } });
}

function uploadButton(): HTMLButtonElement {
	return screen.getByRole<HTMLButtonElement>('button', { name: 'Upload' });
}

describe('UploadFileModal size pre-check', () => {
	beforeEach(() => {
		getMaxUploadBytesMock.mockReset();
	});

	afterEach(() => {
		getModalStore().close();
	});

	it('rejects a file larger than the server limit', async () => {
		getMaxUploadBytesMock.mockResolvedValue(50 * MIB);
		await openModal();

		await selectFile(fileOfSize('big.pdf', 60 * MIB));

		expect(screen.getByRole('alert').textContent).toBe('File is too large. Maximum size is 50 MB.');
		expect(uploadButton().disabled).toBe(true);
	});

	it('accepts a file under the server limit', async () => {
		getMaxUploadBytesMock.mockResolvedValue(50 * MIB);
		await openModal();

		await selectFile(fileOfSize('small.pdf', MIB));

		expect(screen.queryByRole('alert')).toBeNull();
		expect(screen.getByText('small.pdf')).toBeTruthy();
		expect(uploadButton().disabled).toBe(false);
	});

	it('lets the server decide when the limit is unknown', async () => {
		getMaxUploadBytesMock.mockResolvedValue(null);
		await openModal();

		await selectFile(fileOfSize('big.pdf', 60 * MIB));

		expect(screen.queryByRole('alert')).toBeNull();
		expect(screen.getByText('big.pdf')).toBeTruthy();
		expect(uploadButton().disabled).toBe(false);
	});

	it('revalidates a file that was selected before the limit arrived', async () => {
		let resolveLimit: (value: number | null) => void = () => {};
		getMaxUploadBytesMock.mockReturnValue(
			new Promise<number | null>((resolve) => {
				resolveLimit = resolve;
			})
		);
		getModalStore().open('upload');
		render(UploadFileModal);
		await waitFor(() => expect(getMaxUploadBytesMock).toHaveBeenCalled());

		await selectFile(fileOfSize('big.pdf', 60 * MIB));
		expect(screen.queryByRole('alert')).toBeNull();
		expect(uploadButton().disabled).toBe(false);

		resolveLimit(50 * MIB);
		await tick();
		await tick();

		expect(screen.getByRole('alert').textContent).toBe('File is too large. Maximum size is 50 MB.');
		expect(uploadButton().disabled).toBe(true);
	});

	it('reports a fractional limit with one decimal', async () => {
		getMaxUploadBytesMock.mockResolvedValue(1_572_864);
		await openModal();

		await selectFile(fileOfSize('big.pdf', 2 * MIB));

		expect(screen.getByRole('alert').textContent).toBe(
			'File is too large. Maximum size is 1.5 MB.'
		);
	});

	it('reports an unsupported extension before the size check', async () => {
		getMaxUploadBytesMock.mockResolvedValue(50 * MIB);
		await openModal();

		await selectFile(fileOfSize('notes.txt', 60 * MIB, 'text/plain'));

		expect(screen.getByRole('alert').textContent).toContain('Unsupported type');
	});
});
