import assert from 'node:assert/strict'
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'
import test from 'node:test'

const repoRoot = dirname(dirname(fileURLToPath(import.meta.url)))

function run(command, args, options = {}) {
  return spawnSync(command, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    ...options,
  })
}

function packageRelease(tag = 'v0.2.0') {
  const root = mkdtempSync(join(tmpdir(), 'indelible-installer-'))
  const releaseDir = join(root, 'release')
  const result = run('bash', ['scripts/package-self-hosting-release.sh', tag, releaseDir])
  assert.equal(result.status, 0, result.stderr)
  return { root, releaseDir }
}

function installFixture() {
  const fixture = packageRelease()
  const installDir = join(fixture.root, 'install')
  const mockBin = join(fixture.root, 'bin')
  const curlLog = join(fixture.root, 'curl.log')
  const dockerLog = join(fixture.root, 'docker.log')

  mkdirSync(installDir)
  mkdirSync(mockBin)
  copyFileSync(join(fixture.releaseDir, 'install.sh'), join(installDir, 'install.sh'))

  const curlMock = `#!/bin/sh
set -eu
output=
url=
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o)
      output=$2
      shift 2
      ;;
    -*) shift ;;
    *)
      url=$1
      shift
      ;;
  esac
done
test -n "$url"
test -n "$output"
printf '%s\\n' "$url" >> "$MOCK_CURL_LOG"
case "$url" in
  https://github.com/useindelible/indelible/releases/download/v0.2.0/*) ;;
  *) echo "unexpected URL: $url" >&2; exit 22 ;;
esac
cp "$MOCK_RELEASE_DIR/\${url##*/}" "$output"
`
  writeFileSync(join(mockBin, 'curl'), curlMock)
  chmodSync(join(mockBin, 'curl'), 0o755)

  const dockerMock = `#!/bin/sh
set -eu
printf '%s\\n' "$*" >> "$MOCK_DOCKER_LOG"
case "$*" in
  "compose version"|"compose pull"|"compose up -d") ;;
  *) echo "unexpected docker command: $*" >&2; exit 2 ;;
esac
`
  writeFileSync(join(mockBin, 'docker'), dockerMock)
  chmodSync(join(mockBin, 'docker'), 0o755)

  return {
    ...fixture,
    installDir,
    curlLog,
    dockerLog,
    env: {
      ...process.env,
      PATH: `${mockBin}:${process.env.PATH}`,
      MOCK_CURL_LOG: curlLog,
      MOCK_DOCKER_LOG: dockerLog,
      MOCK_RELEASE_DIR: fixture.releaseDir,
    },
  }
}

test('packages a version-pinned installer and includes it in checksums', () => {
  const { releaseDir } = packageRelease('v1.2.3')
  const installer = readFileSync(join(releaseDir, 'install.sh'), 'utf8')
  const checksums = readFileSync(
    join(releaseDir, 'self-hosting-checksums-sha256.txt'),
    'utf8',
  )

  assert.match(installer, /^RELEASE_TAG=v1\.2\.3$/m)
  assert.match(checksums, / install\.sh$/m)
  assert.equal(run('sh', ['-n', join(releaseDir, 'install.sh')]).status, 0)
})

test('installs the exact release, generates secrets, and starts Compose', () => {
  const fixture = installFixture()
  const result = run('sh', ['install.sh'], {
    cwd: fixture.installDir,
    env: fixture.env,
  })

  assert.equal(result.status, 0, result.stderr)
  assert.deepEqual(readFileSync(fixture.curlLog, 'utf8').trim().split('\n'), [
    'https://github.com/useindelible/indelible/releases/download/v0.2.0/docker-compose.yml',
    'https://github.com/useindelible/indelible/releases/download/v0.2.0/example.env',
    'https://github.com/useindelible/indelible/releases/download/v0.2.0/self-hosting-checksums-sha256.txt',
  ])
  assert.deepEqual(readFileSync(fixture.dockerLog, 'utf8').trim().split('\n'), [
    'compose version',
    'compose pull',
    'compose up -d',
  ])

  const env = readFileSync(join(fixture.installDir, '.env'), 'utf8')
  assert.match(env, /^INDELIBLE_VERSION=0\.2\.0$/m)
  assert.match(env, /^POSTGRES_PASSWORD=[0-9a-f]{32}$/m)
  assert.match(env, /^MINIO_ROOT_PASSWORD=[0-9a-f]{32}$/m)
  assert.match(env, /^JWT_SECRET=[0-9a-f]{64}$/m)
  assert.match(env, /^CSRF_SECRET=[0-9a-f]{64}$/m)
  assert.match(env, /^ASSET_COOKIE_SECRET=[0-9a-f]{64}$/m)
  assert.match(env, /^AUTH_CREDENTIAL_KEY=[A-Za-z0-9+/]+={0,2}$/m)
})

test('refuses to overwrite an existing environment file', () => {
  const fixture = installFixture()
  const envPath = join(fixture.installDir, '.env')
  writeFileSync(envPath, 'DO_NOT_REPLACE=true\n')

  const result = run('sh', ['install.sh'], {
    cwd: fixture.installDir,
    env: fixture.env,
  })

  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /\.env already exists/)
  assert.equal(readFileSync(envPath, 'utf8'), 'DO_NOT_REPLACE=true\n')
  assert.equal(existsSync(fixture.curlLog), false)
  assert.equal(existsSync(fixture.dockerLog), false)
})
