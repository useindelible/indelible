import { defineConfig } from 'wxt'
import { fileURLToPath } from 'node:url'

const CHROME_STORE_PUBLIC_KEY =
  'MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAnz12VolmGtYuSdW14kP2' +
  'RwMwgoc0GB+s4T7ViEN6vsNsKse4aISPKvAqPmHXZB/ouYhxgErLvylM+pWdR+RQ' +
  'RXCzyIETfTlSvU4B14EJkYpftbEkcC25iIAxj96XpzvU0yCiPE6v+N5lvxkecgUB' +
  'Sm+qx+draOpF4SkEF8/49eo/qeeHDIxOPOk/VvlYmmVAY+o1HA87BK4kr3sHfuW/' +
  'VSqfmBWevw9ozbsjGK1rgW6sUbE5Q9suYsZ9c7St2CR3+rtciQrW4ka0iMJNGEPx' +
  '3jEvOG0KBEl8Q4a8mFBsZ67cjf8Yh41vcPr+y/hc4zC/vzun/J6Z82GyhHjuESJE' +
  'xQIDAQAB'

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
    excludeSources: [
      '**/*',
    ],
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
    name: 'Indelible',
    description: 'Save, archive, and organize web content with Indelible',
    version: '0.1.0',
    key: browser === 'chrome' ? CHROME_STORE_PUBLIC_KEY : undefined,
    action: {
      default_title: 'Indelible',
    },
    permissions: ['activeTab', 'contextMenus', 'identity', 'scripting', 'storage', 'tabs'],
    commands: {
      'save-current-page': {
        suggested_key: {
          default: 'Alt+Shift+S',
          mac: 'Alt+Shift+S',
        },
        description: 'Save current page to Indelible',
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
