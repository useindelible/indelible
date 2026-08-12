import { tick } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import CreateTagDialog from '../../src/routes/(app)/tags/components/CreateTagDialog.svelte';
import RenameTagDialog from '../../src/routes/(app)/tags/components/RenameTagDialog.svelte';

describe('tag dialogs', () => {
	it('focuses the create input when opened', async () => {
		render(CreateTagDialog, {
			props: {
				color: null,
				name: '',
				parentId: null,
				error: null,
				existingTag: null,
				onClose: vi.fn(),
				onColorChange: vi.fn(),
				onNameChange: vi.fn(),
				onOpenExisting: vi.fn(),
				onSubmit: vi.fn()
			}
		});

		await tick();
		expect(document.activeElement).toBe(screen.getByPlaceholderText(/tag name/i));
	});

	it('focuses the rename input when opened', async () => {
		render(RenameTagDialog, {
			props: {
				value: 'Research',
				onClose: vi.fn(),
				onSubmit: vi.fn(),
				onValueChange: vi.fn()
			}
		});

		await tick();
		expect(document.activeElement).toBe(screen.getByDisplayValue('Research'));
	});
});
