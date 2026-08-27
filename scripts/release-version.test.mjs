import assert from 'node:assert/strict'
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import test from 'node:test'

import { bumpRepository, incrementVersion, inspectRepository } from './release-version.mjs'

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'indelible-release-version-'))
  mkdirSync(join(root, 'backend'), { recursive: true })
  mkdirSync(join(root, 'extension'), { recursive: true })
  mkdirSync(join(root, 'mobile', 'composeApp'), { recursive: true })

  writeFileSync(
    join(root, 'backend', 'Cargo.toml'),
    '[workspace]\nmembers = []\n\n[workspace.package]\nversion = "0.1.0"\nedition = "2024"\n',
  )
  writeFileSync(
    join(root, 'extension', 'package.json'),
    `${JSON.stringify({ name: 'ind-extension', version: '0.1.0' }, null, 2)}\n`,
  )
  writeFileSync(
    join(root, 'extension', 'package-lock.json'),
    `${JSON.stringify(
      {
        name: 'ind-extension',
        version: '0.1.0',
        lockfileVersion: 3,
        packages: { '': { name: 'ind-extension', version: '0.1.0' } },
      },
      null,
      2,
    )}\n`,
  )
  writeFileSync(
    join(root, 'mobile', 'composeApp', 'build.gradle.kts'),
    'android {\n    defaultConfig {\n        versionCode = 1\n        versionName = "0.1.0"\n    }\n}\n',
  )

  return root
}

test('increments stable semantic versions by the requested release type', () => {
  assert.equal(incrementVersion('0.1.0', 'patch'), '0.1.1')
  assert.equal(incrementVersion('0.1.0', 'minor'), '0.2.0')
  assert.equal(incrementVersion('1.9.7', 'major'), '2.0.0')
})

test('rejects malformed versions and unsupported release types', () => {
  assert.throws(() => incrementVersion('v0.1.0', 'patch'), /stable semantic version/)
  assert.throws(() => incrementVersion('0.1.0', 'rc'), /release type/)
})

test('updates every shipped version and increments Android versionCode once', () => {
  const root = fixture()

  const result = bumpRepository(root, 'minor')

  assert.deepEqual(result, { version: '0.2.0', versionCode: 2 })
  assert.deepEqual(inspectRepository(root), { version: '0.2.0', versionCode: 2 })
  assert.match(readFileSync(join(root, 'backend', 'Cargo.toml'), 'utf8'), /version = "0\.2\.0"/)

  const packageLock = JSON.parse(
    readFileSync(join(root, 'extension', 'package-lock.json'), 'utf8'),
  )
  assert.equal(packageLock.version, '0.2.0')
  assert.equal(packageLock.packages[''].version, '0.2.0')
})

test('inspection is non-mutating so recovery does not increment versionCode', () => {
  const root = fixture()
  bumpRepository(root, 'patch')

  const before = readFileSync(join(root, 'mobile', 'composeApp', 'build.gradle.kts'), 'utf8')
  assert.deepEqual(inspectRepository(root), { version: '0.1.1', versionCode: 2 })
  assert.equal(
    readFileSync(join(root, 'mobile', 'composeApp', 'build.gradle.kts'), 'utf8'),
    before,
  )
})

test('refuses mismatched platform versions', () => {
  const root = fixture()
  const packageJsonPath = join(root, 'extension', 'package.json')
  const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf8'))
  packageJson.version = '0.1.1'
  writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`)

  assert.throws(() => inspectRepository(root), /Shipped versions disagree/)
})
