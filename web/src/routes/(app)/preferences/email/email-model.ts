import type {
	AliasDestinationDto,
	DestinationDto,
	EmailAliasResponse,
	EmailSenderResponse
} from '$lib/api';
import type { Translate } from '$lib/i18n';
import { date } from '$lib/i18n';
import { get } from 'svelte/store';

export type SenderFilter = 'all' | 'feed' | 'library' | 'blocked' | 'quiet';

export interface SenderCounts {
	all: number;
	feed: number;
	library: number;
	blocked: number;
	quiet: number;
}

export function primaryAlias(
	aliases: EmailAliasResponse[],
	destination: AliasDestinationDto
): EmailAliasResponse | null {
	const candidates = aliases.filter(
		(alias) =>
			alias.destination === destination &&
			alias.status === 'active' &&
			alias.is_default &&
			!alias.retire_at
	);
	if (candidates.length === 0) return null;
	return candidates.reduce((a, b) => (b.created_at > a.created_at ? b : a));
}

export function isQuiet(sender: EmailSenderResponse): boolean {
	if (sender.blocked || !sender.last_seen_at) return false;
	const diffMs = Date.now() - new Date(sender.last_seen_at).getTime();
	return diffMs > 30 * 24 * 60 * 60 * 1000;
}

export function senderCounts(senders: EmailSenderResponse[]): SenderCounts {
	return {
		all: senders.length,
		feed: senders.filter(
			(sender) =>
				!sender.blocked && (sender.routing_default === 'feed' || sender.routing_default == null)
		).length,
		library: senders.filter((sender) => !sender.blocked && sender.routing_default === 'library')
			.length,
		blocked: senders.filter((sender) => sender.blocked).length,
		quiet: senders.filter((sender) => isQuiet(sender)).length
	};
}

export function filterMatches(sender: EmailSenderResponse, filter: SenderFilter): boolean {
	switch (filter) {
		case 'all':
			return true;
		case 'feed':
			return (
				!sender.blocked && (sender.routing_default === 'feed' || sender.routing_default == null)
			);
		case 'library':
			return !sender.blocked && sender.routing_default === 'library';
		case 'blocked':
			return sender.blocked;
		case 'quiet':
			return isQuiet(sender);
	}
}

export function searchMatches(sender: EmailSenderResponse, query: string): boolean {
	const normalized = query.trim().toLowerCase();
	if (!normalized) return true;
	return (
		sender.canonical_addr.toLowerCase().includes(normalized) ||
		(sender.display_name?.toLowerCase().includes(normalized) ?? false) ||
		(sender.list_id?.toLowerCase().includes(normalized) ?? false)
	);
}

export function filterSenders(
	senders: EmailSenderResponse[],
	filter: SenderFilter,
	query: string
): EmailSenderResponse[] {
	return senders
		.filter((sender) => filterMatches(sender, filter))
		.filter((sender) => searchMatches(sender, query))
		.slice()
		.sort((a, b) => (b.last_seen_at ?? '').localeCompare(a.last_seen_at ?? ''));
}

export function senderInitial(sender: EmailSenderResponse): string {
	const source = sender.display_name?.trim() || sender.canonical_addr;
	return source.charAt(0).toUpperCase();
}

export function formatRelative(iso: string | null | undefined): string {
	if (!iso) return '-';
	const diffMs = Date.now() - new Date(iso).getTime();
	const minutes = Math.floor(diffMs / 60_000);
	if (minutes < 1) return 'just now';
	if (minutes < 60) return `${minutes}m ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `${hours}h ago`;
	const days = Math.floor(hours / 24);
	if (days < 7) return `${days}d ago`;
	const weeks = Math.floor(days / 7);
	if (weeks < 8) return `${weeks}w ago`;
	return get(date)(new Date(iso), {
		month: 'short',
		day: 'numeric',
		year: 'numeric'
	});
}

export function formatIssued(iso: string | null | undefined): string {
	if (!iso) return '-';
	return get(date)(new Date(iso), {
		day: 'numeric',
		month: 'short',
		year: 'numeric'
	});
}

export function domainFromAddress(address: string): string {
	const at = address.indexOf('@');
	return at >= 0 ? address.slice(at) : '';
}

export function routingValue(sender: EmailSenderResponse): string {
	if (sender.routing_default == null) return 'default';
	return sender.routing_default;
}

export function routingPatchValue(raw: string): DestinationDto | null {
	return raw === 'default' ? null : (raw as DestinationDto);
}

export function isValidLocalPart(value: string): boolean {
	const normalized = value.trim().toLowerCase();
	if (normalized.length < 3 || normalized.length > 32) return false;
	if (!/^[a-z0-9._-]+$/.test(normalized)) return false;
	if (normalized.startsWith('.') || normalized.endsWith('.')) return false;
	return true;
}

export function extractErrorMessage(
	apiError: unknown,
	response: Response | undefined,
	fallback: string,
	translate: Translate
): string {
	if (apiError && typeof apiError === 'object') {
		const err = apiError as Record<string, unknown>;
		const fieldErrors = err.field_errors as Array<{ field: string; message: string }> | undefined;
		if (fieldErrors && fieldErrors.length > 0) {
			return fieldErrors.map((field) => `${field.field}: ${field.message}`).join('; ');
		}
		if (typeof err.detail === 'string') return err.detail;
		if (typeof err.message === 'string') return err.message;
	}
	if (response?.status === 409) return translate('email_error_alias_taken');
	if (response?.status === 422) return translate('email_error_invalid_alias');
	return fallback;
}
