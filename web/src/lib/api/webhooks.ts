import {
	createWebhookEndpoint as apiCreateWebhookEndpoint,
	deleteWebhookEndpoint as apiDeleteWebhookEndpoint,
	listWebhookDeliveries as apiListWebhookDeliveries,
	listWebhookEndpoints as apiListWebhookEndpoints,
	rotateWebhookSecret as apiRotateWebhookSecret,
	testWebhookEndpoint as apiTestWebhookEndpoint,
	updateWebhookEndpoint as apiUpdateWebhookEndpoint,
	type CreateWebhookEndpointRequest,
	type UpdateWebhookEndpointRequest,
	type WebhookDeliveryResponse,
	type WebhookEndpointResponse,
	type WebhookEndpointSecretResponse
} from '$lib/api';

type ApiProblem = {
	detail?: string;
	error?: string;
	message?: string;
};

export type WebhookEndpoint = WebhookEndpointResponse;
export type WebhookEndpointWithSecret = WebhookEndpointSecretResponse;
export type WebhookDelivery = WebhookDeliveryResponse;

function extractMessage(problem: unknown, fallback: string): string {
	if (!problem || typeof problem !== 'object') {
		return fallback;
	}

	const candidate = problem as ApiProblem;
	return candidate.detail ?? candidate.message ?? candidate.error ?? fallback;
}

export async function listWebhookEndpoints(): Promise<WebhookEndpoint[]> {
	const { data, error } = await apiListWebhookEndpoints();
	if (data) return data.data;
	throw new Error(extractMessage(error, 'Failed to load webhook endpoints'));
}

export async function createWebhookEndpoint(
	input: CreateWebhookEndpointRequest
): Promise<WebhookEndpointWithSecret> {
	const { data, error } = await apiCreateWebhookEndpoint({ body: input });
	if (data) return data;
	throw new Error(extractMessage(error, 'Failed to create webhook endpoint'));
}

export async function updateWebhookEndpoint(
	id: string,
	patch: UpdateWebhookEndpointRequest
): Promise<WebhookEndpoint> {
	const { data, error } = await apiUpdateWebhookEndpoint({
		path: { webhook_id: id },
		body: patch
	});
	if (data) return data;
	throw new Error(extractMessage(error, 'Failed to update webhook endpoint'));
}

export async function deleteWebhookEndpoint(id: string): Promise<void> {
	const { error } = await apiDeleteWebhookEndpoint({ path: { webhook_id: id } });
	if (!error) return;
	throw new Error(extractMessage(error, 'Failed to delete webhook endpoint'));
}

export async function rotateWebhookSecret(id: string): Promise<WebhookEndpointWithSecret> {
	const { data, error } = await apiRotateWebhookSecret({ path: { webhook_id: id } });
	if (data) return data;
	throw new Error(extractMessage(error, 'Failed to rotate webhook secret'));
}

export async function testWebhookEndpoint(id: string, event: string): Promise<WebhookDelivery> {
	const { data, error } = await apiTestWebhookEndpoint({
		path: { webhook_id: id },
		body: { event }
	});
	if (data) return data;
	throw new Error(extractMessage(error, 'Failed to send test webhook'));
}

export async function listWebhookDeliveries(id: string): Promise<WebhookDelivery[]> {
	const { data, error } = await apiListWebhookDeliveries({ path: { webhook_id: id } });
	if (data) return data.data;
	throw new Error(extractMessage(error, 'Failed to load webhook deliveries'));
}

export const WEBHOOK_EVENT_GROUPS: Array<{
	key: string;
	name: string;
	events: string[];
}> = [
	{
		key: 'library_entry',
		name: 'Library',
		events: [
			'library_entry.saved',
			'library_entry.triaged',
			'library_entry.archived',
			'library_entry.favorited',
			'library_entry.trashed',
			'library_entry.restored',
			'library_entry.permanently_deleted',
			'library_entry.tagged',
			'library_entry.untagged'
		]
	},
	{
		key: 'highlight',
		name: 'Highlights',
		events: ['document.highlighted', 'highlight.updated', 'highlight.deleted', 'highlight.noted']
	},
	{
		key: 'feed',
		name: 'Feeds',
		events: ['feed.subscribed', 'feed.unsubscribed', 'feed.new_item', 'feed.poll_failed']
	},
	{
		key: 'taxonomy',
		name: 'Collections & Tags',
		events: [
			'collection.created',
			'collection.updated',
			'collection.deleted',
			'collection.item_added',
			'collection.item_removed',
			'tag.created',
			'tag.merged'
		]
	},
	{
		key: 'lifecycle',
		name: 'Account, Integrations & Review',
		events: [
			'integration.sync_completed',
			'integration.sync_failed',
			'account.created',
			'account.email_verified',
			'account.deleted',
			'review.completed',
			'review.streak'
		]
	}
];
