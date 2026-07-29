import { describe, expect, it } from 'vitest';

import type { ArchivalSettingsResponse } from '$lib/api/generated/types.gen';
import {
	actionFromApi,
	actionToApi,
	buildArchivalSettingsPayload,
	createArchivalSnapshot,
	getArchiveFormatStatus,
	sensitivityFromApi,
	sensitivityToApi
} from '../../src/routes/(app)/preferences/archival/archival-model';

const serverSettings = (): ArchivalSettingsResponse => ({
	archive_formats: {
		readable_html: true,
		monolith: false,
		pdf: true,
		screenshot: false,
		warc: true
	},
	duplicate_detection: {
		enabled: true,
		sensitivity: 'high',
		on_duplicate: 'merge_with_existing'
	},
	processing: {
		browser_timeout_secs: 120,
		max_concurrent_archives: 4,
		ai_auto_processing: true
	},
	proxy: {
		url: 'https://proxy.example.test',
		all_requests: true
	}
});

describe('archival settings model', () => {
	it('maps duplicate sensitivity values both ways', () => {
		expect(sensitivityFromApi('low')).toBe(1);
		expect(sensitivityFromApi('medium')).toBe(2);
		expect(sensitivityFromApi('high')).toBe(3);
		expect(sensitivityToApi(1)).toBe('low');
		expect(sensitivityToApi(2)).toBe('medium');
		expect(sensitivityToApi(3)).toBe('high');
	});

	it('maps duplicate actions both ways', () => {
		expect(actionFromApi('notify_me')).toBe('notify');
		expect(actionFromApi('skip_silently')).toBe('skip');
		expect(actionFromApi('merge_with_existing')).toBe('merge');
		expect(actionToApi('notify')).toBe('notify_me');
		expect(actionToApi('skip')).toBe('skip_silently');
		expect(actionToApi('merge')).toBe('merge_with_existing');
	});

	it('creates stable snapshots with trimmed proxy URLs', () => {
		expect(
			createArchivalSnapshot({
				formats: { monolith: true, pdf: false, screenshot: true },
				dupEnabled: true,
				dupSensitivity: 2,
				dupAction: 'notify',
				proxyUrl: '  socks5://127.0.0.1:1080  ',
				proxyAll: true
			})
		).toBe(
			JSON.stringify({
				formats: { monolith: true, pdf: false, screenshot: true },
				dupEnabled: true,
				dupSensitivity: 2,
				dupAction: 'notify',
				proxyUrl: 'socks5://127.0.0.1:1080',
				proxyAll: true
			})
		);
	});

	it('builds the save payload without dropping server-owned fields', () => {
		expect(
			buildArchivalSettingsPayload({
				serverData: serverSettings(),
				formats: { monolith: true, pdf: false, screenshot: true },
				dupEnabled: false,
				dupSensitivity: 1,
				dupAction: 'skip',
				proxyUrl: '  ',
				proxyAll: true
			})
		).toEqual({
			archive_formats: {
				readable_html: true,
				monolith: true,
				pdf: false,
				screenshot: true,
				warc: true
			},
			duplicate_detection: {
				enabled: false,
				sensitivity: 'low',
				on_duplicate: 'skip_silently'
			},
			processing: {
				browser_timeout_secs: 120,
				max_concurrent_archives: 4,
				ai_auto_processing: true
			},
			proxy: {
				url: null,
				all_requests: false
			}
		});
	});

	it('reports visual format status from metadata and enabled flags', () => {
		expect(
			getArchiveFormatStatus('readable', { monolith: false, pdf: false, screenshot: false })
		).toBe('on');
		expect(getArchiveFormatStatus('warc', { monolith: true, pdf: true, screenshot: true })).toBe(
			'coming'
		);
		expect(getArchiveFormatStatus('pdf', { monolith: true, pdf: false, screenshot: true })).toBe(
			'off'
		);
		expect(
			getArchiveFormatStatus('screenshot', { monolith: false, pdf: false, screenshot: true })
		).toBe('on');
	});
});
