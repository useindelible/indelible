import { defineConfig } from 'wxt'

export default defineConfig({
  srcDir: '.',
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
    action: {
      default_title: 'Indelible',
    },
    permissions: ['activeTab', 'contextMenus', 'scripting', 'storage', 'tabs'],
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
