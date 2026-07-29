import { build } from 'esbuild'
import { readFile, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { dirname, resolve } from 'node:path'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const outputPath = resolve(root, '../backend/apps/ind-renderer/src/dom-preprocessor.js')
const result = await build({
  entryPoints: [resolve(root, 'lib/dom-preprocessor.ts')],
  bundle: true,
  format: 'iife',
  globalName: 'IndelibleDomPreprocessor',
  minify: true,
  platform: 'browser',
  target: ['chrome120', 'firefox140'],
  write: false,
})
const output = `// Generated from extension/lib/dom-preprocessor.ts; run npm run dom-preprocessor:build.\n${result.outputFiles[0].text}`

if (process.argv.includes('--check')) {
  const current = await readFile(outputPath, 'utf8').catch(() => '')
  if (current !== output) {
    console.error('Renderer DOM preprocessor bundle is stale. Run npm run dom-preprocessor:build.')
    process.exitCode = 1
  }
} else {
  await writeFile(outputPath, output)
}
