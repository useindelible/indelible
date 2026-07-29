import * as apiSdk from '$lib/api';
import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';

export async function openSearchResult(result: SearchResultResponse): Promise<void> {
	if (result.document_id) {
		await goto(resolve('/(app)/reader/[documentId]', { documentId: result.document_id }));
		return;
	}
	if (!result.delivery_id) return;
	try {
		const { data } = await apiSdk.prepareFeedDelivery({
			path: { delivery_id: result.delivery_id }
		});
		if (data?.document_id) {
			await goto(resolve('/(app)/reader/[documentId]', { documentId: data.document_id }));
		}
	} catch {
		// Preparation failed (e.g. a no-URL delivery); leave the result in place.
	}
}
