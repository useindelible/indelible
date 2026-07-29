/**
 * OAuth callback parsing for the integrations hub.
 *
 * The backend redirects browser navigations to:
 *   - `/preferences/integrations?connected={provider}` on success
 *   - `/preferences/integrations?integration_error={kind}&provider={p}` on failure
 *
 * `kind` is one of `denied | provider_error | server`. The hub mounts
 * `IntegrationCallbackBanner` to render a tone-appropriate banner from these
 * params, then drops the params via `replaceState` so a refresh stays idempotent.
 */

export type IntegrationCallbackKind = 'success' | 'denied' | 'provider_error' | 'server_error';

export interface IntegrationCallback {
	kind: IntegrationCallbackKind;
	provider: string | null;
}

export function parseIntegrationCallback(url: URL): IntegrationCallback | null {
	const params = url.searchParams;
	const connected = params.get('connected');
	if (connected) {
		return { kind: 'success', provider: connected };
	}
	const errorKind = params.get('integration_error');
	if (!errorKind) return null;
	const provider = params.get('provider');
	switch (errorKind) {
		case 'denied':
			return { kind: 'denied', provider };
		case 'provider_error':
			return { kind: 'provider_error', provider };
		case 'server':
			return { kind: 'server_error', provider };
		default:
			// Unknown kinds collapse to the generic server-error surface — better
			// than swallowing them and leaving the user with no signal.
			return { kind: 'server_error', provider };
	}
}
