import { flushSync } from 'svelte';
import { describe, expect, it } from 'vitest';

import type { ScopeLayer } from './dispatch';
import { activeLayers, registerShortcuts } from './registry.svelte';

function mountLayer(layer: ScopeLayer): () => void {
	const destroy = $effect.root(() => {
		registerShortcuts(() => layer);
	});
	flushSync();
	return destroy;
}

describe('registerShortcuts', () => {
	it('publishes a layer while its owner is mounted', () => {
		const destroy = mountLayer({ scope: 'library', handlers: {} });
		expect(activeLayers()).toHaveLength(1);

		destroy();
		flushSync();
		expect(activeLayers()).toHaveLength(0);
	});

	it('keeps the incoming layer when the outgoing one unmounts late', () => {
		const outgoingLayer: ScopeLayer = { scope: 'library', handlers: {} };
		const incomingLayer: ScopeLayer = { scope: 'library', handlers: {} };

		const outgoing = mountLayer(outgoingLayer);
		const incoming = mountLayer(incomingLayer);
		expect(activeLayers()).toHaveLength(2);

		outgoing();
		flushSync();

		expect([...activeLayers()]).toEqual([incomingLayer]);

		incoming();
		flushSync();
		expect(activeLayers()).toHaveLength(0);
	});

	it('publishes nothing while the builder declines', () => {
		const destroy = $effect.root(() => {
			registerShortcuts(() => null);
		});
		flushSync();

		expect(activeLayers()).toHaveLength(0);
		destroy();
	});
});
