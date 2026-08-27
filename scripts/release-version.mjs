#!/usr/bin/env node

import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const stableVersion = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/
const releaseTypes = new Set(['patch', 'minor', 'major'])

function read(path) {
  return readFileSync(path, 'utf8')
}

function replaceExactlyOnce(text, pattern, replacement, label) {
  const matches = [...text.matchAll(pattern)]
  if (matches.length !== 1) {
    throw new Error(`Expected exactly one ${label}, found ${matches.length}`)
  }
  return text.replace(pattern, replacement)
}

function parseStableVersion(version) {
  const match = stableVersion.exec(version)
  if (!match) throw new Error(`Expected a stable semantic version, got "${version}"`)
  return match.slice(1).map(Number)
}

export function incrementVersion(version, releaseType) {
  if (!releaseTypes.has(releaseType)) {
    throw new Error(`Unsupported release type "${releaseType}"; use patch, minor, or major`)
  }

  let [major, minor, patch] = parseStableVersion(version)
  if (releaseType === 'major') {
    major += 1
    minor = 0
    patch = 0
  } else if (releaseType === 'minor') {
    minor += 1
    patch = 0
  } else {
    patch += 1
  }
  return `${major}.${minor}.${patch}`
}

function cargoWorkspaceVersion(text) {
  const section = text.match(/\[workspace\.package\]\n([\s\S]*?)(?=\n\[|$)/)
  if (!section) throw new Error('Missing [workspace.package] in backend/Cargo.toml')
  const versions = [...section[1].matchAll(/^version\s*=\s*"([^"]+)"$/gm)]
  if (versions.length !== 1) {
    throw new Error(`Expected one workspace package version, found ${versions.length}`)
  }
  return versions[0][1]
}

function gradleValue(text, name, valuePattern) {
  const matches = [...text.matchAll(new RegExp(`^\\s*${name}\\s*=\\s*${valuePattern}$`, 'gm'))]
  if (matches.length !== 1) throw new Error(`Expected one Android ${name}, found ${matches.length}`)
  return matches[0][1]
}

function paths(root) {
  return {
    cargo: join(root, 'backend', 'Cargo.toml'),
    extensionPackage: join(root, 'extension', 'package.json'),
    extensionLock: join(root, 'extension', 'package-lock.json'),
    android: join(root, 'mobile', 'composeApp', 'build.gradle.kts'),
  }
}

function readState(root) {
  const repositoryPaths = paths(root)
  const cargoText = read(repositoryPaths.cargo)
  const extensionPackageText = read(repositoryPaths.extensionPackage)
  const extensionLockText = read(repositoryPaths.extensionLock)
  const androidText = read(repositoryPaths.android)
  const extensionPackage = JSON.parse(extensionPackageText)
  const extensionLock = JSON.parse(extensionLockText)

  const versions = {
    backend: cargoWorkspaceVersion(cargoText),
    extension: extensionPackage.version,
    extensionLock: extensionLock.version,
    extensionLockRoot: extensionLock.packages?.['']?.version,
    android: gradleValue(androidText, 'versionName', '"([^"]+)"'),
  }
  const versionCode = Number(gradleValue(androidText, 'versionCode', '(\\d+)'))

  return {
    repositoryPaths,
    texts: { cargoText, extensionPackageText, extensionLockText, androidText },
    parsed: { extensionPackage, extensionLock },
    versions,
    versionCode,
  }
}

export function inspectRepository(root) {
  const state = readState(root)
  const distinctVersions = new Set(Object.values(state.versions))
  if (distinctVersions.size !== 1) {
    throw new Error(
      `Shipped versions disagree: ${Object.entries(state.versions)
        .map(([name, version]) => `${name}=${version}`)
        .join(', ')}`,
    )
  }

  const [version] = distinctVersions
  parseStableVersion(version)
  if (!Number.isSafeInteger(state.versionCode) || state.versionCode < 1) {
    throw new Error(`Android versionCode must be a positive integer, got ${state.versionCode}`)
  }
  return { version, versionCode: state.versionCode }
}

export function bumpRepository(root, releaseType) {
  const current = inspectRepository(root)
  const nextVersion = incrementVersion(current.version, releaseType)
  const state = readState(root)

  const cargoText = replaceExactlyOnce(
    state.texts.cargoText,
    /^(version\s*=\s*)"[^"]+"$/gm,
    `$1"${nextVersion}"`,
    'backend workspace version',
  )
  const androidText = replaceExactlyOnce(
    replaceExactlyOnce(
      state.texts.androidText,
      /^(\s*versionName\s*=\s*)"[^"]+"$/gm,
      `$1"${nextVersion}"`,
      'Android versionName',
    ),
    /^(\s*versionCode\s*=\s*)\d+$/gm,
    `$1${current.versionCode + 1}`,
    'Android versionCode',
  )

  state.parsed.extensionPackage.version = nextVersion
  state.parsed.extensionLock.version = nextVersion
  if (!state.parsed.extensionLock.packages?.['']) {
    throw new Error('extension/package-lock.json is missing its root package')
  }
  state.parsed.extensionLock.packages[''].version = nextVersion

  writeFileSync(state.repositoryPaths.cargo, cargoText)
  writeFileSync(
    state.repositoryPaths.extensionPackage,
    `${JSON.stringify(state.parsed.extensionPackage, null, 2)}\n`,
  )
  writeFileSync(
    state.repositoryPaths.extensionLock,
    `${JSON.stringify(state.parsed.extensionLock, null, 2)}\n`,
  )
  writeFileSync(state.repositoryPaths.android, androidText)

  return inspectRepository(root)
}

function usage() {
  console.error('usage: release-version.mjs inspect [repository-root]')
  console.error('       release-version.mjs bump <patch|minor|major> [repository-root]')
}

function main() {
  const [command, argument, optionalRoot] = process.argv.slice(2)
  const defaultRoot = join(dirname(fileURLToPath(import.meta.url)), '..')

  if (command === 'inspect' && optionalRoot === undefined) {
    console.log(JSON.stringify(inspectRepository(argument ?? defaultRoot)))
    return
  }
  if (command === 'bump' && argument && process.argv.length <= 5) {
    console.log(JSON.stringify(bumpRepository(optionalRoot ?? defaultRoot, argument)))
    return
  }
  usage()
  process.exitCode = 2
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main()
