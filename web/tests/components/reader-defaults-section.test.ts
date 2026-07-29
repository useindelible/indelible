import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import ReaderDefaultsSection from '../../src/routes/(app)/preferences/reading-appearance/components/ReaderDefaultsSection.svelte';
import { FONT_SIZE_LABEL } from '../../src/routes/(app)/preferences/reading-appearance/reading-appearance-model';

describe('ReaderDefaultsSection', () => {
	it('renders reader defaults and emits callback props', async () => {
		const onFontFamilyChange = vi.fn();
		const onFontSizeChange = vi.fn();
		const onLineHeightChange = vi.fn();

		render(ReaderDefaultsSection, {
			props: {
				fontFamily: 'serif',
				fontSize: 'medium',
				lineHeight: 'relaxed',
				fontSizeLabel: FONT_SIZE_LABEL,
				onFontFamilyChange,
				onFontSizeChange,
				onLineHeightChange
			}
		});

		expect(screen.getByText('Reader defaults')).toBeTruthy();
		expect(screen.getByText('Medium')).toBeTruthy();

		await fireEvent.click(screen.getByRole('radio', { name: 'Sans' }));
		await fireEvent.click(screen.getByLabelText('Increase size'));
		await fireEvent.click(screen.getByRole('radio', { name: /compact/i }));

		expect(onFontFamilyChange).toHaveBeenCalledWith('sans');
		expect(onFontSizeChange).toHaveBeenCalledWith('large');
		expect(onLineHeightChange).toHaveBeenCalledWith('compact');
	});
});
