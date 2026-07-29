// Copies SingleFile hook scripts from node_modules into public/single-file/
// so they can be referenced as web-accessible resources and injected via
// browser.scripting.executeScript({ files: [...] }).
//
// SingleFile ships as plain JS files, not ES modules, so they cannot be
// imported directly through the bundler.

import { copyFileSync, mkdirSync, existsSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const publicDir = resolve(dirname(new URL(import.meta.url).pathname), '..', 'public', 'single-file')

if (!existsSync(publicDir)) {
  mkdirSync(publicDir, { recursive: true })
}

// single-file-core has no "main" entry, so resolve via package.json
const singleFilePkg = resolve(dirname(require.resolve('single-file-core/package.json')))

// Injection order matters: bootstrap sets up the environment, then single-file.js uses it.
const filesToCopy = [
  ['single-file-bootstrap.js', 'single-file-bootstrap.js'],
  ['single-file.js', 'single-file.js'],
  ['single-file-frames.js', 'single-file-frames.js'],
  ['processors/hooks/content/content-hooks-frames.js', 'content-hooks-frames.js'],
  ['processors/hooks/content/content-hooks-frames-web.js', 'content-hooks-frames-web.js'],
]

let copied = 0
for (const [src, dest] of filesToCopy) {
  const srcPath = resolve(singleFilePkg, src)
  const destPath = resolve(publicDir, dest)
  if (existsSync(srcPath)) {
    copyFileSync(srcPath, destPath)
    copied++
  } else {
    console.warn(`[copy-singlefile] Not found, skipping: ${srcPath}`)
  }
}

console.log(`[copy-singlefile] Copied ${copied}/${filesToCopy.length} files to public/single-file/`)
