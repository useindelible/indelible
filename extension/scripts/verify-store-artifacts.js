import { execFileSync } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const root = process.cwd()
const packageJson = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'))
const version = packageJson.version
const requiredFirefoxDataTypes = [
  'authenticationInfo',
  'browsingActivity',
  'websiteActivity',
  'websiteContent',
]

function run(command, args, options = {}) {
  execFileSync(command, args, {
    cwd: root,
    stdio: options.capture ? ['ignore', 'pipe', 'inherit'] : 'inherit',
    encoding: 'utf8',
  })
}

function readManifest(target) {
  const path = join(root, '.output', target, 'manifest.json')
  if (!existsSync(path)) {
    throw new Error(`Missing manifest for ${target}: ${path}`)
  }
  return JSON.parse(readFileSync(path, 'utf8'))
}

function assert(condition, message) {
  if (!condition) throw new Error(message)
}

function assertMv3Manifest(target, manifest) {
  assert(manifest.manifest_version === 3, `${target} must be Manifest V3`)
  assert(!manifest.action?.default_popup, `${target} must not declare default_popup`)
  assert(manifest.browser_action === undefined, `${target} must not declare browser_action`)
  assert(manifest.page_action === undefined, `${target} must not declare page_action`)
  assert(manifest.optional_host_permissions === undefined, `${target} must not use optional hosts`)
  assert(manifest.host_permissions === undefined, `${target} must not declare host permissions`)
  assert(manifest.content_scripts === undefined, `${target} must not auto-register content scripts`)
  assert(
    Array.isArray(manifest.web_accessible_resources) &&
      manifest.web_accessible_resources.length === 1,
    `${target} must only expose SingleFile web-accessible resources`,
  )
  const resources = manifest.web_accessible_resources[0]?.resources
  assert(
    Array.isArray(resources) && resources.length === 1 && resources[0] === 'single-file/*.js',
    `${target} web-accessible resources must be limited to single-file/*.js`,
  )
}

function assertFirefoxManifest(manifest) {
  const gecko = manifest.browser_specific_settings?.gecko
  const geckoAndroid = manifest.browser_specific_settings?.gecko_android
  assert(gecko?.id === 'extension@useindelible.com', 'Firefox gecko.id is missing')
  assert(gecko.strict_min_version === '140.0', 'Firefox strict_min_version must be 140.0')
  assert(
    geckoAndroid?.strict_min_version === '142.0',
    'Firefox Android strict_min_version must be 142.0',
  )

  const required = gecko.data_collection_permissions?.required
  assert(Array.isArray(required), 'Firefox data_collection_permissions.required is missing')
  for (const dataType of requiredFirefoxDataTypes) {
    assert(required.includes(dataType), `Firefox data collection type is missing: ${dataType}`)
  }
}

function assertZipExists(browser) {
  const path = join(root, '.output', `ind-extension-${version}-${browser}.zip`)
  assert(existsSync(path), `Missing ${browser} store zip: ${path}`)
}

function assertFirefoxLint() {
  const output = execFileSync(
    'npx',
    ['--yes', 'web-ext', 'lint', '--source-dir', '.output/firefox-mv3', '--output', 'json'],
    { cwd: root, encoding: 'utf8' },
  )
  const report = JSON.parse(output)
  assert(report.summary?.errors === 0, `Firefox lint reported ${report.summary?.errors} errors`)
  const warningCodes = new Set(
    (report.warnings ?? []).map((warning) => warning.code).filter(Boolean),
  )
  assert(
    !warningCodes.has('MISSING_DATA_COLLECTION_PERMISSIONS'),
    'Firefox lint reports missing data_collection_permissions',
  )
  assert(!warningCodes.has('MISSING_ADDON_ID'), 'Firefox lint reports missing gecko.id')
}

run('npm', ['run', 'build'])
run('npm', ['run', 'build:edge'])
run('npm', ['run', 'build:firefox'])
run('npm', ['run', 'zip:all'])

const chromeManifest = readManifest('chrome-mv3')
const edgeManifest = readManifest('edge-mv3')
const firefoxManifest = readManifest('firefox-mv3')

assertMv3Manifest('chrome-mv3', chromeManifest)
assertMv3Manifest('edge-mv3', edgeManifest)
assertMv3Manifest('firefox-mv3', firefoxManifest)
assertFirefoxManifest(firefoxManifest)

assertZipExists('chrome')
assertZipExists('edge')
assertZipExists('firefox')
assert(
  existsSync(join(root, '.output', `ind-extension-${version}-sources.zip`)),
  'Missing source zip',
)
assertFirefoxLint()

console.log('Store artifacts verified: chrome-mv3, edge-mv3, firefox-mv3')
