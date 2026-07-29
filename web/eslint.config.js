import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import svelteRunes from 'eslint-plugin-svelte-runes';
import prettier from 'eslint-config-prettier';
import globals from 'globals';

export default ts.config(
	js.configs.recommended,
	...ts.configs.recommended,
	...svelte.configs['flat/recommended'],
	prettier,
	...svelte.configs['flat/prettier'],
	{
		languageOptions: {
			globals: {
				...globals.browser,
				...globals.node
			}
		}
	},
	{
		files: ['**/*.svelte'],
		languageOptions: {
			parserOptions: {
				parser: ts.parser,
				svelteFeatures: {
					runes: true
				}
			}
		}
	},
	{
		files: ['**/*.svelte.ts'],
		languageOptions: {
			parserOptions: {
				parser: ts.parser,
				svelteFeatures: {
					runes: true
				}
			}
		}
	},
	// eslint-plugin-svelte-runes 0.0.11 uses legacy config format;
	// manually adapted to flat config here
	{
		plugins: { 'svelte-runes': svelteRunes },
		rules: svelteRunes.configs.recommended.rules
	},
	{
		files: ['src/**/*.{ts,svelte}'],
		ignores: ['src/lib/api/**'],
		rules: {
			'no-restricted-imports': [
				'error',
				{
					patterns: [
						{
							group: [
								'$lib/api/generated',
								'$lib/api/generated/client.gen',
								'$lib/api/generated/sdk.gen',
								'**/api/generated',
								'**/api/generated/client.gen',
								'**/api/generated/sdk.gen'
							],
							allowTypeImports: true,
							message: 'Use the typed service exports from $lib/api.'
						}
					]
				}
			],
			'no-restricted-syntax': [
				'error',
				{
					selector: 'ImportExpression[source.value=/api\\/generated(?:\\/(?:client|sdk)\\.gen)?$/]',
					message: 'Dynamically import generated clients only inside $lib/api.'
				}
			]
		}
	},
	{
		ignores: ['build/', '.svelte-kit/', 'node_modules/', 'coverage/', 'src/lib/api/generated/**']
	}
);
