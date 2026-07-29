import tseslint from 'typescript-eslint'

export default tseslint.config(
  ...tseslint.configs.strict,
  {
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
    },
  },
  {
    files: ['tests/**/*.ts'],
    rules: {
      '@typescript-eslint/no-non-null-assertion': 'off',
    },
  },
  {
    files: ['entrypoints/**/*.ts', 'components/**/*.ts', 'lib/**/*.ts'],
    ignores: ['lib/api.ts', 'lib/api/**'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: [
                '@/lib/api/generated',
                '@/lib/api/generated/client.gen',
                '@/lib/api/generated/sdk.gen',
                '**/api/generated',
                '**/api/generated/client.gen',
                '**/api/generated/sdk.gen',
              ],
              allowTypeImports: true,
              message: 'Use the typed service exports from @/lib/api.',
            },
          ],
        },
      ],
      'no-restricted-syntax': [
        'error',
        {
          selector: 'ImportExpression[source.value=/api\\/generated(?:\\/(?:client|sdk)\\.gen)?$/]',
          message: 'Dynamically import generated clients only inside @/lib/api.',
        },
      ],
    },
  },
  {
    ignores: ['.output/', '.wxt/', 'public/single-file/', 'lib/api/generated/**'],
  },
)
