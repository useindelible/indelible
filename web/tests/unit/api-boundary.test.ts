import { describe, expect, it } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const srcRoot = join(projectRoot, 'src');

function sourceFiles(dir: string): string[] {
	return readdirSync(dir).flatMap((name) => {
		const path = join(dir, name);
		const stat = statSync(path);
		if (stat.isDirectory()) {
			return sourceFiles(path);
		}
		if (/\.(svelte|ts)$/.test(name)) {
			return [path];
		}
		return [];
	});
}

describe('API generated client boundary', () => {
	it('keeps generated client and SDK value imports inside src/lib/api', () => {
		const offenders = sourceFiles(srcRoot)
			.filter((file) => !relative(srcRoot, file).startsWith('lib/api/'))
			.flatMap((file) => {
				const source = readFileSync(file, 'utf8');
				const matches = source.matchAll(
					/import\s+(?!type\b)[\s\S]*?from\s+['"]\$lib\/api\/generated\/(?:client|sdk)\.gen['"]/g
				);
				return [...matches].map(() => relative(projectRoot, file));
			});

		expect(offenders).toEqual([]);
	});
});
