import type { IntegrationConnectionDto } from '$lib/api';

export type ConnectionState = 'connected' | 'disconnected' | 'syncing' | 'failed' | 'unavailable';

export function deriveConnectionState(
	connection: IntegrationConnectionDto | undefined
): ConnectionState {
	if (!connection) return 'disconnected';
	switch (connection.status) {
		case 'active':
			return 'connected';
		case 'syncing':
			return 'syncing';
		case 'error':
			return 'failed';
		case 'paused':
		case 'disabled':
			return 'unavailable';
		default:
			return 'unavailable';
	}
}

export type ImportStatus =
	| 'awaiting_provider'
	| 'pending'
	| 'running'
	| 'completed'
	| 'failed'
	| 'partial'
	| 'rolled_back'
	| 'unknown';

export function normalizeImportStatus(status: string): ImportStatus {
	switch (status) {
		case 'awaiting_provider':
		case 'pending':
		case 'running':
		case 'completed':
		case 'failed':
		case 'partial':
		case 'rolled_back':
			return status;
		default:
			return 'unknown';
	}
}

export function isPollingStatus(status: string): boolean {
	const normalized = normalizeImportStatus(status);
	return normalized === 'awaiting_provider' || normalized === 'pending' || normalized === 'running';
}

export function isTerminalImportStatus(status: string): boolean {
	const normalized = normalizeImportStatus(status);
	return (
		normalized === 'completed' ||
		normalized === 'failed' ||
		normalized === 'partial' ||
		normalized === 'rolled_back'
	);
}

// The backend exposes opaque `last_error` text rather than structured failure details.
// These heuristics keep recoverable-failure messaging visible without changing the wire shape.
export function detectRateLimit(lastError: string | null | undefined): boolean {
	if (!lastError) return false;
	const lower = lastError.toLowerCase();
	return lower.includes('429') || lower.includes('retry-after') || lower.includes('rate limit');
}

export function detectAuthFailure(lastError: string | null | undefined): boolean {
	if (!lastError) return false;
	const lower = lastError.toLowerCase();
	return lower.includes('401') || lower.includes('unauthorized');
}
