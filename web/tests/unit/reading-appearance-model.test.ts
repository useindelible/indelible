import { describe, expect, it } from 'vitest';
import type { PreferencesSettingsResponse } from '$lib/api';
import {
	buildPreferencesSaveBody,
	bumpFontSize,
	draftFromPreferences,
	readerPreviewStyles,
	readingAppearanceSnapshot
} from '../../src/routes/(app)/preferences/reading-appearance/reading-appearance-model';

function preferences(
	overrides: Partial<PreferencesSettingsResponse> = {}
): PreferencesSettingsResponse {
	return {
		theme: 'system',
		appearance: { accent_color: 'blue' },
		layout: {
			sidebar_mode: 'expanded',
			default_view: 'library',
			list_density: 'compact',
			side_panel: 'open'
		},
		workflow: { triage_mode: 'focus', auto_advance: true },
		reader: {
			font_family: 'serif',
			font_size: 'medium',
			line_height: 'relaxed',
			email_open_mode: 'reader'
		},
		ai: { mila_enabled: true, custom_prompt: null },
		...overrides
	};
}

describe('reading appearance model', () => {
	it('creates a draft and stable dirty snapshot from preferences', () => {
		const draft = draftFromPreferences(preferences(), 'en-US');

		expect(draft.locale).toBe('en-US');
		expect(readingAppearanceSnapshot(draft)).toEqual(
			JSON.stringify({
				theme: 'system',
				accentColor: 'blue',
				sidebarMode: 'expanded',
				defaultView: 'library',
				listDensity: 'compact',
				sidePanel: 'open',
				triageMode: 'focus',
				autoAdvance: true,
				fontFamily: 'serif',
				fontSize: 'medium',
				lineHeight: 'relaxed',
				emailOpenMode: 'reader',
				locale: 'en-US'
			})
		);
	});

	it('builds save bodies while preserving server ai settings', () => {
		const draft = draftFromPreferences(
			preferences({ ai: { mila_enabled: false, custom_prompt: 'Be terse.' } }),
			'en-GB'
		);
		draft.theme = 'dark';
		draft.emailOpenMode = 'original';

		expect(buildPreferencesSaveBody(draft, preferences()).reader.email_open_mode).toBe('original');
		expect(buildPreferencesSaveBody(draft, preferences()).theme).toBe('dark');
		expect(buildPreferencesSaveBody(draft, preferences()).ai).toEqual({
			mila_enabled: true,
			custom_prompt: null
		});
	});

	it('derives preview styles and bounded font-size steps', () => {
		expect(readerPreviewStyles('mono', 'large', 'compact')).toEqual({
			fontFamily: "'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace",
			fontSize: '17px',
			lineHeight: '1.42'
		});
		expect(bumpFontSize('small', -1)).toBe('small');
		expect(bumpFontSize('medium', 1)).toBe('large');
		expect(bumpFontSize('large', 1)).toBe('large');
	});
});
