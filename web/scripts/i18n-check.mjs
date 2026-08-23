#!/usr/bin/env node

import { readFile, readdir } from 'node:fs/promises';
import { relative, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { IntlMessageFormat } from 'intl-messageformat';
import { parseTree } from 'jsonc-parser';
import { parse } from 'svelte/compiler';
import ts from 'typescript';

const VISIBLE_ATTRIBUTES = new Set(['alt', 'aria-label', 'placeholder', 'title']);
const UI_PROPERTY_NAMES = new Set([
	'ariaLabel',
	'copy',
	'description',
	'emptyText',
	'error',
	'hint',
	'label',
	'message',
	'placeholder',
	'title'
]);
const UI_FUNCTION_NAME = /(Copy|Description|Hint|Label|Message|Status|Text|Title)$/;
export const ALLOWED_KEY_PREFIXES = [
	'account_',
	'archival_',
	'auth_',
	'collection_',
	'common_',
	'dashboard_',
	'email_',
	'entity_',
	'error_',
	'extension_',
	'feed_',
	'imports_',
	'integrations_',
	'layout_',
	'library_',
	'mila_',
	'onboarding_',
	'prefs_',
	'reader_',
	'search_',
	'settings_',
	'smart_',
	'tag_',
	'trash_'
];
const PRODUCT_COPY = new Set([
	'Anthropic',
	'API',
	'Atom',
	'EPUB',
	'GitHub',
	'Gmail',
	'Chrome',
	'Edge',
	'Firefox',
	'Indelible',
	'JSON',
	'Mila',
	'Notion',
	'OAuth',
	'Obsidian',
	'Ollama',
	'OpenAI',
	'OPML',
	'PDF',
	'Readwise',
	'Readwise Reader',
	'RSS',
	'Safari',
	'Postroom',
	'URL',
	'Web',
	'X',
	'YouTube'
]);

function collectArguments(elements, argumentsFound = new Set()) {
	for (const element of elements) {
		if (element.type !== 0 && element.type !== 7 && typeof element.value === 'string') {
			argumentsFound.add(element.value);
		}
		if (element.options) {
			for (const option of Object.values(element.options)) {
				collectArguments(option.value, argumentsFound);
			}
		}
		if (element.children) collectArguments(element.children, argumentsFound);
	}
	return argumentsFound;
}

async function readCatalog(file, locale, errors, allowedPrefixes) {
	const source = await readFile(file, 'utf8');
	const parseErrors = [];
	const root = parseTree(source, parseErrors, {
		allowTrailingComma: false,
		disallowComments: true
	});
	if (parseErrors.length > 0 || root?.type !== 'object') {
		errors.push(`${locale}: catalog must be a valid JSON object`);
		return new Map();
	}

	const messages = new Map();
	let previousKey;
	for (const property of root.children ?? []) {
		const [keyNode, valueNode] = property.children ?? [];
		const key = keyNode?.value;
		if (typeof key !== 'string') continue;
		if (allowedPrefixes.length > 0 && !allowedPrefixes.some((prefix) => key.startsWith(prefix))) {
			errors.push(`${locale}.${key}: disallowed key prefix`);
		}

		if (messages.has(key)) errors.push(`${locale}.${key}: duplicate key`);
		if (previousKey !== undefined && previousKey > key) {
			errors.push(`${locale}: keys are not sorted: ${previousKey} must follow ${key}`);
		}
		previousKey = key;

		if (valueNode?.type !== 'string' || typeof valueNode.value !== 'string') {
			errors.push(`${locale}.${key}: value must be a string`);
			continue;
		}
		if (valueNode.value.trim() === '') {
			errors.push(`${locale}.${key}: value must be non-empty`);
		}

		let argumentsFound = new Set();
		try {
			argumentsFound = collectArguments(new IntlMessageFormat(valueNode.value, locale).getAst());
		} catch (error) {
			const detail = error instanceof Error ? error.message : String(error);
			errors.push(`${locale}.${key}: invalid ICU message (${detail})`);
		}
		messages.set(key, { arguments: argumentsFound });
	}
	return messages;
}

function sameSet(left, right) {
	return left.size === right.size && [...left].every((value) => right.has(value));
}

function lineNumber(source, offset) {
	return source.slice(0, offset).split('\n').length;
}

function normalizeCopy(value) {
	return value.replace(/\s+/g, ' ').trim();
}

function isProductCopy(value) {
	const parts = normalizeCopy(value).split(/\s*(?:×|·|—|\/|&|\+)\s*/);
	return (
		parts.length > 0 &&
		parts.every((part) =>
			[...PRODUCT_COPY].some((product) => product.toLowerCase() === part.toLowerCase())
		)
	);
}

function looksLikeUserCopy(value, visibleContext = false) {
	const text = normalizeCopy(value);
	if (!/\p{L}/u.test(text) || isProductCopy(text)) return false;
	if ([...text].length === 1) return false;
	if (/^[A-Z]{2,}$/.test(text) || /^(?:Aa|List-ID)$/.test(text)) return false;
	if (/^(?:em|fixed|px)$/i.test(text)) return false;
	if (/^[·\s]*(?:CSV|OPML|ZIP)$/i.test(text)) return false;
	if (/^(?:\.[a-z0-9]+)(?:\s+\.[a-z0-9]+)*$/i.test(text)) return false;
	if (/^[a-z][a-z0-9_-]*:"/.test(text) || /^[A-Za-z]+Error$/.test(text)) return false;
	if (!/\s/u.test(text) && /[:/.@_-]/.test(text)) return false;
	return visibleContext || /\s/u.test(text) || /^\p{Lu}/u.test(text);
}

function reportSourceLiteral(
	errors,
	filename,
	source,
	offset,
	value,
	kind,
	visibleContext = false
) {
	const text = normalizeCopy(value);
	if (!looksLikeUserCopy(text, visibleContext)) return;
	errors.push(`${filename}:${lineNumber(source, offset)}: raw ${kind} "${text}"`);
}

function checkSvelteSource(source, filename, errors) {
	let ast;
	try {
		ast = parse(source, { filename, modern: true });
	} catch {
		return;
	}

	function visit(node) {
		if (!node || typeof node !== 'object') return;
		if (Array.isArray(node)) {
			for (const child of node) visit(child);
			return;
		}
		if (node.type === 'Text') {
			reportSourceLiteral(errors, filename, source, node.start, node.data, 'template text', true);
			return;
		}
		if (node.type === 'RegularElement' && ['script', 'style'].includes(node.name)) return;
		if (typeof node.type === 'string' && node.type.endsWith('Directive')) return;
		if (node.type === 'Attribute') {
			if (VISIBLE_ATTRIBUTES.has(node.name) && Array.isArray(node.value)) {
				for (const value of node.value) {
					if (value.type === 'Text') {
						reportSourceLiteral(
							errors,
							filename,
							source,
							value.start,
							value.data,
							`${node.name} attribute`,
							true
						);
					}
				}
			}
			return;
		}
		if (node.type === 'ExpressionTag') return;

		for (const [key, value] of Object.entries(node)) {
			if (['attributes', 'expression', 'metadata', 'name_loc'].includes(key)) continue;
			visit(value);
		}
		visit(node.attributes);
	}

	visit(ast.fragment);
}

function functionName(node) {
	if (node.name && ts.isIdentifier(node.name)) return node.name.text;
	if (node.parent && ts.isVariableDeclaration(node.parent) && ts.isIdentifier(node.parent.name)) {
		return node.parent.name.text;
	}
	return '';
}

function checkTypeScriptSource(source, filename, errors) {
	const ast = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true);

	function visit(node, uiFunction = false) {
		let nextUiFunction = uiFunction;
		if (
			ts.isFunctionDeclaration(node) ||
			ts.isFunctionExpression(node) ||
			ts.isArrowFunction(node) ||
			ts.isMethodDeclaration(node)
		) {
			nextUiFunction = UI_FUNCTION_NAME.test(functionName(node));
		}

		if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) {
			const parent = node.parent;
			const propertyName =
				parent && ts.isPropertyAssignment(parent) && ts.isIdentifier(parent.name)
					? parent.name.text
					: '';
			const translated =
				parent &&
				ts.isCallExpression(parent) &&
				ts.isIdentifier(parent.expression) &&
				['message', 't', 'translate'].includes(parent.expression.text);
			if (!translated && (nextUiFunction || UI_PROPERTY_NAMES.has(propertyName))) {
				reportSourceLiteral(
					errors,
					filename,
					source,
					node.getStart(ast),
					node.text,
					'TypeScript UI text'
				);
			}
		}

		ts.forEachChild(node, (child) => visit(child, nextUiFunction));
	}

	visit(ast);
}

async function sourceFiles(directory) {
	const files = [];
	for (const entry of await readdir(directory, { withFileTypes: true })) {
		if (entry.name === 'generated') continue;
		const path = resolve(directory, entry.name);
		if (entry.isDirectory()) files.push(...(await sourceFiles(path)));
		else if (entry.name.endsWith('.svelte') || entry.name.endsWith('.ts')) files.push(path);
	}
	return files;
}

async function checkSources(sourcesDir, errors) {
	for (const file of await sourceFiles(sourcesDir)) {
		if (/\.(?:test|spec)\.(?:svelte\.)?ts$/.test(file)) continue;
		const source = await readFile(file, 'utf8');
		const filename = relative(process.cwd(), file);
		if (file.endsWith('.svelte')) checkSvelteSource(source, filename, errors);
		else checkTypeScriptSource(source, filename, errors);
	}
}

export async function checkCatalogs({
	localesDir,
	referenceLocales = ['fr'],
	sourcesDir,
	allowedPrefixes = []
}) {
	const errors = [];
	const summary = {};
	const filenames = (await readdir(localesDir)).filter((file) => file.endsWith('.json')).sort();
	const catalogs = new Map();

	for (const filename of filenames) {
		const locale = filename.slice(0, -'.json'.length);
		catalogs.set(
			locale,
			await readCatalog(resolve(localesDir, filename), locale, errors, allowedPrefixes)
		);
	}

	const english = catalogs.get('en');
	if (!english) {
		errors.push('en: reference catalog is missing');
		return { errors, summary };
	}

	for (const [locale, catalog] of catalogs) {
		for (const [key, message] of catalog) {
			const reference = english.get(key);
			if (!reference) {
				errors.push(`${locale}.${key}: unknown key absent from en`);
				continue;
			}
			if (locale !== 'en' && !sameSet(reference.arguments, message.arguments)) {
				errors.push(`${locale}.${key}: arguments do not match en`);
			}
		}

		const missing = [...english.keys()].filter((key) => !catalog.has(key));
		summary[locale] = { total: english.size, missing: missing.length };
		if (referenceLocales.includes(locale)) {
			for (const key of missing) errors.push(`${locale}.${key}: missing reference translation`);
		}
	}

	for (const locale of referenceLocales) {
		if (!catalogs.has(locale)) errors.push(`${locale}: reference catalog is missing`);
	}

	if (sourcesDir) await checkSources(sourcesDir, errors);

	return { errors, summary };
}

async function main() {
	const result = await checkCatalogs({
		localesDir: resolve('src/lib/i18n/locales'),
		sourcesDir: resolve('src'),
		allowedPrefixes: ALLOWED_KEY_PREFIXES
	});
	for (const [locale, stats] of Object.entries(result.summary)) {
		console.log(`${locale}: ${stats.total - stats.missing}/${stats.total} translated`);
	}
	for (const error of result.errors) console.error(error);
	if (result.errors.length > 0) process.exitCode = 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
	await main();
}
