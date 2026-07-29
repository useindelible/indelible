import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import SearchResultRow from '$lib/components/search/SearchResultRow.svelte';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';

function baseResult(overrides: Partial<SearchResultResponse> = {}): SearchResultResponse {
	return {
		document_id: 'doc_test',
		result_kind: 'document',
		title: 'Test Newsletter',
		snippet: 'Body snippet',
		score: 0.5,
		content_type: 'email',
		saved_at: new Date('2026-05-18T10:00:00Z').toISOString(),
		updated_at: new Date('2026-05-18T10:00:00Z').toISOString(),
		...overrides
	};
}

describe('SearchResultRow sender chip', () => {
	it('does not render a sender chip when the result has no sender', () => {
		render(SearchResultRow, {
			props: {
				result: baseResult(),
				selected: false,
				onSelect: () => {},
				onOpen: () => {},
				onSenderClick: () => {}
			}
		});
		expect(screen.queryByTestId('search-sender-chip')).toBeNull();
	});

	it('renders the sender display_name when present', () => {
		render(SearchResultRow, {
			props: {
				result: baseResult({
					sender_id: 'snd_abc',
					sender: {
						id: 'snd_abc',
						canonical_addr: 'news@example.com',
						display_name: 'Example Daily',
						blocked: false
					}
				}),
				selected: false,
				onSelect: () => {},
				onOpen: () => {},
				onSenderClick: () => {}
			}
		});
		const chip = screen.getByTestId('search-sender-chip');
		expect(chip.textContent).toContain('Example Daily');
	});

	it('falls back to canonical_addr when display_name is missing', () => {
		render(SearchResultRow, {
			props: {
				result: baseResult({
					sender_id: 'snd_xyz',
					sender: {
						id: 'snd_xyz',
						canonical_addr: 'news@example.com',
						blocked: false
					}
				}),
				selected: false,
				onSelect: () => {},
				onOpen: () => {},
				onSenderClick: () => {}
			}
		});
		const chip = screen.getByTestId('search-sender-chip');
		expect(chip.textContent).toContain('news@example.com');
	});

	it('invokes onSenderClick with canonical_addr and stops propagation to row open', async () => {
		const onOpen = vi.fn();
		const onSenderClick = vi.fn();
		render(SearchResultRow, {
			props: {
				result: baseResult({
					sender_id: 'snd_abc',
					sender: {
						id: 'snd_abc',
						canonical_addr: 'news@example.com',
						display_name: 'Example Daily',
						blocked: false
					}
				}),
				selected: false,
				onSelect: () => {},
				onOpen,
				onSenderClick
			}
		});
		const chip = screen.getByTestId('search-sender-chip');
		await fireEvent.click(chip);
		expect(onSenderClick).toHaveBeenCalledTimes(1);
		expect(onSenderClick).toHaveBeenCalledWith('news@example.com');
		expect(onOpen).not.toHaveBeenCalled();
	});
});
