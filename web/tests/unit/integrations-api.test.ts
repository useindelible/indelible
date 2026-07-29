import { describe, it, expect, vi, beforeEach } from 'vitest';

const sdkMocks = vi.hoisted(() => ({
	listIntegrations: vi.fn(),
	authorizeIntegration: vi.fn(),
	deleteIntegration: vi.fn(),
	syncIntegration: vi.fn(),
	setupObsidianConnection: vi.fn()
}));

vi.mock('$lib/api', () => ({
	listIntegrations: sdkMocks.listIntegrations,
	authorizeIntegration: sdkMocks.authorizeIntegration,
	deleteIntegration: sdkMocks.deleteIntegration,
	syncIntegration: sdkMocks.syncIntegration,
	setupObsidianConnection: sdkMocks.setupObsidianConnection
}));

import {
	dispatchIntegrationSync,
	disconnectIntegration,
	loadIntegrationConnections,
	setupObsidianExportConnection,
	startIntegrationAuthorization
} from '$lib/api/integrations';

describe('loadIntegrationConnections', () => {
	beforeEach(() => {
		Object.values(sdkMocks).forEach((m) => m.mockReset());
	});

	it('returns success with the data on a 200', async () => {
		sdkMocks.listIntegrations.mockResolvedValueOnce({
			data: { connections: [] },
			error: undefined,
			response: new Response(null, { status: 200 })
		});
		const result = await loadIntegrationConnections();
		expect(result).toEqual({ success: true, data: { connections: [] } });
	});

	it('extracts the API problem detail on error', async () => {
		sdkMocks.listIntegrations.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'forbidden' },
			response: new Response(null, { status: 403 })
		});
		const result = await loadIntegrationConnections();
		expect(result).toEqual({ success: false, error: 'forbidden' });
	});

	it('falls back to a generic error when the SDK throws', async () => {
		sdkMocks.listIntegrations.mockRejectedValueOnce(new Error('network'));
		const result = await loadIntegrationConnections();
		expect(result.success).toBe(false);
		if (!result.success) expect(result.error).toBe('network');
	});
});

describe('startIntegrationAuthorization', () => {
	beforeEach(() => {
		Object.values(sdkMocks).forEach((m) => m.mockReset());
	});

	it('passes provider and redirect_after to the SDK', async () => {
		sdkMocks.authorizeIntegration.mockResolvedValueOnce({
			data: { authorization_url: 'https://example.com/oauth' },
			error: undefined,
			response: new Response(null, { status: 200 })
		});

		const result = await startIntegrationAuthorization('notion', '/preferences/integrations');
		expect(sdkMocks.authorizeIntegration).toHaveBeenCalledWith({
			path: { provider: 'notion' },
			body: { redirect_after: '/preferences/integrations' }
		});
		expect(result.success).toBe(true);
	});

	it('passes null when redirect_after is omitted', async () => {
		sdkMocks.authorizeIntegration.mockResolvedValueOnce({
			data: { authorization_url: 'https://example.com' },
			error: undefined,
			response: new Response(null, { status: 200 })
		});
		await startIntegrationAuthorization('notion');
		expect(sdkMocks.authorizeIntegration).toHaveBeenCalledWith({
			path: { provider: 'notion' },
			body: { redirect_after: null }
		});
	});

	it('returns a failure with provider context when the SDK errors', async () => {
		sdkMocks.authorizeIntegration.mockResolvedValueOnce({
			data: undefined,
			error: {},
			response: new Response(null, { status: 500 })
		});
		const result = await startIntegrationAuthorization('notion');
		expect(result.success).toBe(false);
		if (!result.success) expect(result.error).toMatch(/notion/);
	});
});

describe('dispatchIntegrationSync', () => {
	beforeEach(() => {
		Object.values(sdkMocks).forEach((m) => m.mockReset());
	});

	it('returns success with sync response on a 202', async () => {
		sdkMocks.syncIntegration.mockResolvedValueOnce({
			data: { job_id: 'job_1' },
			error: undefined,
			response: new Response(null, { status: 202 })
		});
		const result = await dispatchIntegrationSync('cnx_1');
		expect(result).toEqual({
			success: true,
			data: { job_id: 'job_1' }
		});
	});

	it('returns the API error message on failure', async () => {
		sdkMocks.syncIntegration.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'rate-limited' },
			response: new Response(null, { status: 429 })
		});
		const result = await dispatchIntegrationSync('cnx_1');
		expect(result).toEqual({ success: false, error: 'rate-limited' });
	});
});

describe('disconnectIntegration', () => {
	beforeEach(() => {
		Object.values(sdkMocks).forEach((m) => m.mockReset());
	});

	it('treats a 204 response as success', async () => {
		sdkMocks.deleteIntegration.mockResolvedValueOnce({
			data: undefined,
			error: undefined,
			response: new Response(null, { status: 204 })
		});
		const result = await disconnectIntegration('cnx_1');
		expect(result.success).toBe(true);
	});

	it('returns failure for non-2xx responses', async () => {
		sdkMocks.deleteIntegration.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'not yours' },
			response: new Response(null, { status: 403 })
		});
		const result = await disconnectIntegration('cnx_1');
		expect(result).toEqual({ success: false, error: 'not yours' });
	});
});

describe('setupObsidianExportConnection', () => {
	beforeEach(() => {
		Object.values(sdkMocks).forEach((m) => m.mockReset());
	});

	it('creates or returns an Obsidian export connection', async () => {
		const connectionPayload = {
			id: 'icn_1',
			provider: 'obsidian',
			status: 'active',
			last_sync_at: null,
			last_error: null,
			config: { provider: 'obsidian' },
			pending_jobs: 0,
			created_at: '2026-04-25T12:00:00Z'
		};
		sdkMocks.setupObsidianConnection.mockResolvedValueOnce({
			data: connectionPayload,
			error: undefined,
			response: new Response(null, { status: 200 })
		});
		const result = await setupObsidianExportConnection();
		expect(sdkMocks.setupObsidianConnection).toHaveBeenCalledWith();
		expect(result).toEqual({ success: true, data: connectionPayload });
	});

	it('surfaces the failure message verbatim', async () => {
		sdkMocks.setupObsidianConnection.mockResolvedValueOnce({
			data: undefined,
			error: { message: 'limit reached' },
			response: new Response(null, { status: 422 })
		});
		const result = await setupObsidianExportConnection();
		expect(result).toEqual({ success: false, error: 'limit reached' });
	});
});
