import * as generated from './generated';

export async function trashCount() {
	const { data } = await generated.listLibraryTrash({ query: { limit: 1 } });
	return { data: { count: data?.data.length ?? 0 } };
}

export async function itemTypeCounts() {
	return { data: { counts: {} as Record<string, number> } };
}
