/**
 * Renders the built site and reports the failures a build cannot catch:
 * page errors, horizontal overflow, reveals that never fired, and screen
 * frames that did not scale into their column.
 *
 * Usage:  node scripts/check-page.mjs [path]        (default "/")
 * Serves ./dist on PORT (8907) and writes screenshots to .shots/.
 *
 * Playwright lives in the sibling ../web workspace rather than here — this
 * site has no test dependencies of its own and does not need any.
 */
import { chromium } from '../../web/node_modules/@playwright/test/index.mjs';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';
import { mkdirSync } from 'node:fs';

const ROUTE = process.argv[2] || '/';
const PORT = Number(process.env.PORT || 8907);
const OUT = process.env.OUT || '.shots';
const DIST = new URL('../dist/', import.meta.url).pathname;

const TYPES = {
	'.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript',
	'.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.woff': 'font/woff',
	'.json': 'application/json', '.png': 'image/png', '.ico': 'image/x-icon',
};

const server = createServer(async (req, res) => {
	let p = normalize(decodeURIComponent(req.url.split('?')[0]));
	if (p.endsWith('/')) p += 'index.html';
	try {
		const body = await readFile(join(DIST, p));
		res.writeHead(200, { 'content-type': TYPES[extname(p)] || 'application/octet-stream' });
		res.end(body);
	} catch {
		res.writeHead(404).end('not found');
	}
});
await new Promise((r) => server.listen(PORT, r));

const CASES = [
	{ name: 'dark', width: 1440, height: 900, theme: 'dark' },
	{ name: 'light', width: 1440, height: 900, theme: 'light' },
	{ name: 'mobile', width: 390, height: 844, theme: 'dark' },
];

mkdirSync(OUT, { recursive: true });
const browser = await chromium.launch();
let bad = 0;

for (const c of CASES) {
	const ctx = await browser.newContext({
		viewport: { width: c.width, height: c.height },
		deviceScaleFactor: 2,
		colorScheme: c.theme,
	});
	const page = await ctx.newPage();
	const errors = [];
	page.on('pageerror', (e) => errors.push('pageerror: ' + e.message));
	page.on('requestfailed', (r) => {
		if (!r.url().startsWith('data:')) errors.push('requestfailed: ' + r.url());
	});

	await page.goto(`http://localhost:${PORT}${ROUTE}`, { waitUntil: 'networkidle' });

	// Reveals are IntersectionObserver-driven, so the page has to actually
	// travel. scroll-behavior:smooth makes successive scrollTo calls fight, so
	// each step is instant.
	await page.evaluate(async () => {
		const step = Math.round(innerHeight * 0.6);
		for (let y = 0; y <= document.documentElement.scrollHeight; y += step) {
			window.scrollTo({ top: y, behavior: 'instant' });
			await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
		}
		window.scrollTo({ top: 0, behavior: 'instant' });
		await new Promise((r) => setTimeout(r, 1200));
	});

	const m = await page.evaluate(() => {
		const hidden = [...document.querySelectorAll('[data-rev]')]
			.filter((e) => parseFloat(getComputedStyle(e).opacity) < 0.05)
			.map((e) => (e.className || e.tagName) + '|' + e.dataset.rev);

		const de = document.documentElement;
		const overflowX = de.scrollWidth > de.clientWidth + 1;
		let widest = null;
		if (overflowX) {
			for (const el of document.querySelectorAll('*')) {
				const r = el.getBoundingClientRect();
				if (r.right > de.clientWidth + 1 && (!widest || r.right > widest.right)) {
					widest = { cls: String(el.className || el.tagName).slice(0, 46), right: Math.round(r.right) };
				}
			}
		}

		// A frame whose inner stage did not scale leaves dead space; report the
		// worst offender as a percentage of the frame's own height.
		let deadest = null;
		for (const f of document.querySelectorAll('.screen')) {
			const inner = f.querySelector('.screen-inner');
			if (!inner) continue;
			const fr = f.getBoundingClientRect();
			const ir = inner.getBoundingClientRect();
			if (fr.height < 4) continue;
			const dead = Math.round(((fr.height - ir.height) / fr.height) * 100);
			if (!deadest || dead > deadest.dead) {
				deadest = { dead, d: f.dataset.d, cls: String(f.className).slice(0, 40) };
			}
		}

		// Unstyled elements: a class that matched no rule at all.
		return { hidden, overflowX, widest, deadest, height: de.scrollHeight };
	});

	await page.screenshot({ path: `${OUT}/${c.name}.png`, fullPage: true });

	const fail = errors.length || m.overflowX || m.hidden.length || (m.deadest?.dead ?? 0) > 2;
	if (fail) bad++;
	console.log(
		`${fail ? 'FAIL' : 'ok  '} ${c.name.padEnd(7)} h=${m.height} ` +
			`errors=${errors.length || 'none'} overflowX=${m.overflowX} ` +
			`hidden=${m.hidden.length} deadspace=${m.deadest ? m.deadest.dead + '%' : 'n/a'}`,
	);
	if (errors.length) console.log('   ', errors.slice(0, 5).join('\n    '));
	if (m.widest) console.log('    widest:', JSON.stringify(m.widest));
	if (m.hidden.length) console.log('    hidden:', m.hidden.slice(0, 6).join(' , '));
	if ((m.deadest?.dead ?? 0) > 2) console.log('    deadspace:', JSON.stringify(m.deadest));

	await ctx.close();
}

await browser.close();
server.close();
process.exit(bad ? 1 : 0);
