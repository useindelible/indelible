import { afterEach, describe, it, expect, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import SearchResultList from '$lib/components/search/SearchResultList.svelte';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';
import { locale, setupI18nSync } from '$lib/i18n';
import fr from '$lib/i18n/locales/fr.json';

afterEach(() => {
	void locale.set('en');
});

function durableResult(): SearchResultResponse {
	return {
		document_id: 'doc_durable',
		result_kind: 'document',
		title: 'Durable Document',
		snippet: 'Saved content',
		score: 0.9,
		content_type: 'article',
		saved_at: '2026-05-18T10:00:00Z',
		updated_at: '2026-05-18T10:00:00Z'
	};
}

function previewResult(): SearchResultResponse {
	return {
		document_id: null,
		delivery_id: 'dlv_preview',
		source_entry_id: 'fse_1',
		result_kind: 'feed_preview',
		title: 'Feed Preview Story',
		snippet: 'Unprepared delivery',
		score: 0.7,
		content_type: 'article',
		saved_at: '2026-05-18T10:00:00Z',
		updated_at: '2026-05-18T10:00:00Z'
	};
}

function renderList(results: SearchResultResponse[], onOpen: (r: SearchResultResponse) => void) {
	render(SearchResultList, {
		props: {
			results,
			loading: false,
			loadingMore: false,
			hasMore: false,
			isEmpty: false,
			selectedId: null,
			query: '',
			onLoadMore: () => {},
			onSelect: () => {},
			onOpen
		}
	});
}

describe('SearchResultList onOpen wiring', () => {
	it('passes the full durable result object to onOpen', async () => {
		const onOpen = vi.fn();
		renderList([durableResult()], onOpen);

		await fireEvent.click(screen.getByText('Durable Document').closest('[role="option"]')!);

		expect(onOpen).toHaveBeenCalledTimes(1);
		const arg = onOpen.mock.calls[0][0] as SearchResultResponse;
		expect(arg.document_id).toBe('doc_durable');
	});

	it('passes the full feed_preview result object (delivery_id, null document_id) to onOpen', async () => {
		const onOpen = vi.fn();
		renderList([previewResult()], onOpen);

		await fireEvent.click(screen.getByText('Feed Preview Story').closest('[role="option"]')!);

		expect(onOpen).toHaveBeenCalledTimes(1);
		const arg = onOpen.mock.calls[0][0] as SearchResultResponse;
		expect(arg.document_id).toBeNull();
		expect(arg.delivery_id).toBe('dlv_preview');
	});
});

describe('SearchResultList localization', () => {
	it('renders the empty state in the active locale', () => {
		setupI18nSync({ fr }, 'fr');
		render(SearchResultList, {
			props: {
				results: [],
				loading: false,
				loadingMore: false,
				hasMore: false,
				isEmpty: true,
				selectedId: null,
				query: '',
				onLoadMore: () => {},
				onSelect: () => {},
				onOpen: () => {}
			}
		});

		expect(screen.getByText('Aucun résultat trouvé')).toBeTruthy();
	});
});
