import { describe, it, expect } from 'vitest';
import { parseIntegrationCallback } from '$lib/integrations/callback';

function url(query: string): URL {
	return new URL(`https://app.example.com/preferences/integrations${query}`);
}

describe('parseIntegrationCallback', () => {
	it('returns null when no integration params are present', () => {
		expect(parseIntegrationCallback(url(''))).toBeNull();
		expect(parseIntegrationCallback(url('?foo=bar'))).toBeNull();
	});

	it('returns success with provider when ?connected=… is present', () => {
		expect(parseIntegrationCallback(url('?connected=notion'))).toEqual({
			kind: 'success',
			provider: 'notion'
		});
	});

	it('returns denied when integration_error=denied', () => {
		expect(parseIntegrationCallback(url('?integration_error=denied&provider=notion'))).toEqual({
			kind: 'denied',
			provider: 'notion'
		});
	});

	it('returns provider_error for integration_error=provider_error', () => {
		expect(
			parseIntegrationCallback(url('?integration_error=provider_error&provider=notion'))
		).toEqual({ kind: 'provider_error', provider: 'notion' });
	});

	it('returns server_error for integration_error=server', () => {
		expect(parseIntegrationCallback(url('?integration_error=server&provider=notion'))).toEqual({
			kind: 'server_error',
			provider: 'notion'
		});
	});

	it('collapses unknown integration_error kinds to server_error', () => {
		expect(parseIntegrationCallback(url('?integration_error=banana&provider=notion'))).toEqual({
			kind: 'server_error',
			provider: 'notion'
		});
	});

	it('tolerates a missing provider on errors', () => {
		expect(parseIntegrationCallback(url('?integration_error=denied'))).toEqual({
			kind: 'denied',
			provider: null
		});
	});

	it('prefers ?connected over ?integration_error when both are present', () => {
		expect(
			parseIntegrationCallback(url('?connected=notion&integration_error=denied&provider=notion'))
		).toEqual({ kind: 'success', provider: 'notion' });
	});
});
