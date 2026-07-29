# Security Policy

Indelible is an open-source, self-hostable read-it-later and knowledge archiver.
We take the security of both the hosted service and self-hosted deployments
seriously.

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report privately through either channel:

- **GitHub Security Advisories** (preferred): open a private report at
  <https://github.com/realsama/indelible/security/advisories/new>.
- **Email**: security@useindelible.com.

Please include: affected component (backend API, worker, renderer, web,
extension, mobile), a description of the issue, reproduction steps or a
proof-of-concept, and the impact you believe it has. If you have a suggested
fix, include it.

### Response targets

| Stage | Target |
|-------|--------|
| Acknowledgement of your report | within 3 business days |
| Initial assessment + severity triage | within 7 business days |
| Fix or mitigation for Critical/High issues | within 30 days of triage |

We will keep you updated through the advisory thread and credit you in the
release notes unless you ask to remain anonymous. Please give us a reasonable
window to ship a fix before any public disclosure (coordinated disclosure).

## Supported Versions

Indelible is pre-1.0 and ships from `main`. Security fixes land on `main` and
in the latest tagged release. Older tags do not receive backports; self-hosters
should track the latest release. Once 1.0 ships, this policy will be updated to
cover the current and previous minor versions.

| Version | Supported |
|---------|-----------|
| latest release / `main` | ✅ |
| older tags | ❌ |

## Security Model & Known Limitations

These are deliberate properties of the current design, disclosed so operators
and users can make informed decisions.

- **Authentication is single-factor.** v1 supports password and OAuth/OIDC
  sign-in but does **not** offer a second factor (TOTP, WebAuthn/passkeys, or
  SMS). Multi-factor authentication is on the roadmap. Operators handling
  sensitive archives should place the deployment behind an SSO/IdP that enforces
  MFA, and users should use a strong, unique password and a hardware-backed
  OAuth provider where possible.
- **Tenant isolation is enforced at the query layer.** Every data access is
  scoped to the authenticated `user_id`; there is no cross-tenant admin surface
  in v1.
- **Outbound fetches are guarded** against SSRF (private/loopback/link-local/
  metadata ranges blocked, DNS-rebinding-resistant) for the ingestion, feed,
  webhook, and AI-provider surfaces. The headless renderer pre-flights the
  navigation URL but cannot intercept page-initiated subresource requests; run
  the renderer container without a route to internal/metadata networks. See
  `docs/security/` for the egress and self-host hardening notes.
- **Untrusted document content** (saved articles, emails, EPUBs, uploads) is
  sanitized server-side before storage and rendering, and is fenced as
  untrusted data in AI prompts. Treat third-party Obsidian/Notion plugins that
  consume exported content as able to execute arbitrary vault content.
- **Self-hosting requires operator action**: terminate TLS at a reverse proxy,
  set unique secrets, and apply the security headers documented in
  `docs/security/self-host-security-headers.md`. Browser extensions request
  broad host permissions at capture time; see `extension/PERMISSIONS.md`.

## Scope

In scope: the backend (`ind-api`, `ind-worker`, `ind-renderer`), the web app,
the browser extension, the mobile app, and the OSS deployment artifacts.

Out of scope: findings that require a pre-compromised host or device, social
engineering, denial-of-service via volumetric traffic (mitigate at your reverse
proxy/CDN), and issues in third-party dependencies already tracked upstream
(report those to the dependency, and to us if Indelible's usage is the vector).
