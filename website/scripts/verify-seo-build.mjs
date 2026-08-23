import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const redirects = await readFile(new URL('../dist/_redirects', import.meta.url), 'utf8');
assert.match(
	redirects,
	/^\/docs\/?\s+\/docs\/getting-started\/introduction\/\s+301$/m,
	'Cloudflare redirect for /docs/ is missing',
);

const home = await readFile(new URL('../dist/index.html', import.meta.url), 'utf8');
assert.match(
	home,
	/href="\/docs\/getting-started\/introduction\/"/,
	'the Docs navigation must link directly to the canonical page',
);

const sitemap = await readFile(new URL('../dist/sitemap-0.xml', import.meta.url), 'utf8');
assert.doesNotMatch(
	sitemap,
	/<loc>https:\/\/useindelible\.com\/screens\/<\/loc>/,
	'/screens/ must not be included in the sitemap',
);
assert.match(
	sitemap,
	/<loc>https:\/\/useindelible\.com\/docs\/getting-started\/introduction\/<\/loc>/,
	'the canonical docs entry point must remain in the sitemap',
);
