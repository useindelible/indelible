import { defineConfig } from 'wxt'
import { fileURLToPath } from 'node:url'

const CHROME_STORE_PUBLIC_KEY =
  'MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1TQE0zDHwZ6vDBU5FHlL' +
  'tCleak+O1WiNLvUVPesaxdCkX6K7qk4fL+tGMOS8rEvr/UWrYumhj1aTT8hD+i48' +
  'RBK97OkLxu9pMIFvWO1RHi7gLprB3aEexDyxLgjYANIPUoKzoTi2BnOgGdnmUffx' +
  'bO5jQdrPkmBZ3Mn1mXsZcaz6kH4j8lbc9BQZupUeUyPeFvbfzTM45ZcDHl3M6hCG' +
  '0ebID3iJw8v1Dwj5aCBpur/tHIw0AR3C8tQRsayLYUjpRS4Mlxo/v0rx5c1CzkYB' +
  'NyqRXcR3WJoBOi+3ztq+gQy0cTvLwr0lmXJOyXkLdcN+iIt31bF0GCEPlHOBRi4w' +
  'FwIDAQAB'

export default defineConfig({
  srcDir: '.',
  zip: {
    sourcesRoot: fileURLToPath(new URL('..', import.meta.url)),
    includeSources: [
      'extension/*',
      'extension/entrypoints/**',
      'extension/lib/**',
      'extension/public/**',
      'extension/scripts/**',
      'extension/tests/**',
      'shared/highlight-source.ts',
    ],
    excludeSources: ['**/*'],
  },
  dev: {
    server: { port: 3457 },
  },
  hooks: {
    'build:manifestGenerated': (_, manifest) => {
      if (Array.isArray(manifest.content_scripts) && manifest.content_scripts.length === 0) {
        delete manifest.content_scripts
      }
    },
  },
  manifest: ({ browser }) => ({
    default_locale: 'en',
    name: '__MSG_ext_name__',
    description: '__MSG_ext_description__',
    key: browser === 'chrome' ? CHROME_STORE_PUBLIC_KEY : undefined,
    action: {
      default_title: '__MSG_action_title__',
    },
    permissions: ['activeTab', 'contextMenus', 'identity', 'scripting', 'storage', 'tabs'],
    commands: {
      'save-current-page': {
        suggested_key: {
          default: 'Alt+Shift+S',
          mac: 'Alt+Shift+S',
        },
        description: '__MSG_command_save_page__',
      },
    },
    browser_specific_settings:
      browser === 'firefox'
        ? {
            gecko: {
              id: 'extension@useindelible.com',
              strict_min_version: '140.0',
              data_collection_permissions: {
                required: [
                  'authenticationInfo',
                  'browsingActivity',
                  'websiteContent',
                  'websiteActivity',
                ],
              },
            },
            gecko_android: {
              strict_min_version: '142.0',
            },
          }
        : undefined,
    // SingleFile hook scripts are packaged locally and exposed for SingleFile's frame capture path.
    web_accessible_resources: [
      {
        resources: ['single-file/*.js'],
        matches: ['<all_urls>'],
        use_dynamic_url: true,
      },
    ],
  }),
})
