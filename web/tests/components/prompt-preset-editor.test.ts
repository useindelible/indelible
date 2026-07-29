import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import PromptPresetEditor from '../../src/routes/(app)/preferences/ai/components/PromptPresetEditor.svelte';

describe('PromptPresetEditor', () => {
	it('edits prompt preset fields and saves', async () => {
		const onChange = vi.fn();
		const onSave = vi.fn();

		render(PromptPresetEditor, {
			props: {
				actionName: 'Summary',
				editor: {
					mode: 'add',
					action: 'summary',
					name: 'Brief',
					system_prompt: 'Summarize briefly.',
					is_default: false
				},
				editorSaving: false,
				onCancel: vi.fn(),
				onChange,
				onSave
			}
		});

		await fireEvent.input(screen.getByLabelText('Preset name'), {
			target: { value: 'Tight TL;DR' }
		});
		expect(onChange).toHaveBeenCalledWith({ name: 'Tight TL;DR' });

		await fireEvent.input(screen.getByLabelText('System prompt'), {
			target: { value: 'Use three bullets.' }
		});
		expect(onChange).toHaveBeenCalledWith({ system_prompt: 'Use three bullets.' });

		await fireEvent.click(screen.getByRole('switch', { name: /set as default/i }));
		expect(onChange).toHaveBeenCalledWith({ is_default: true });

		await fireEvent.click(screen.getByRole('button', { name: /add preset/i }));
		expect(onSave).toHaveBeenCalledOnce();
	});
});
