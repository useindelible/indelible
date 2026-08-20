import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const savePillSource = readFileSync(
	resolve(process.cwd(), 'src/lib/components/settings/SavePill.svelte'),
	'utf8'
);
const settingsLayoutSource = readFileSync(
	resolve(process.cwd(), 'src/routes/(app)/preferences/+layout.svelte'),
	'utf8'
);

describe('SavePill', () => {
	it('stays anchored to the bottom of the current settings viewport', () => {
		expect(savePillSource).toMatch(
			/\.save-pill-anchor\s*\{[^}]*position:\s*fixed;[^}]*right:\s*0;[^}]*bottom:\s*20px;[^}]*width:\s*100cqw;/s
		);
		expect(settingsLayoutSource).toMatch(
			/\.settings-content\s*\{[^}]*container-type:\s*inline-size;/s
		);
	});

	it('does not leave invisible controls interactive', () => {
		expect(savePillSource).toMatch(
			/\.save-pill\s*\{[^}]*pointer-events:\s*none;[^}]*visibility:\s*hidden;/s
		);
		expect(savePillSource).toMatch(
			/\.save-pill\.visible\s*\{[^}]*pointer-events:\s*auto;[^}]*visibility:\s*visible;/s
		);
	});
});
