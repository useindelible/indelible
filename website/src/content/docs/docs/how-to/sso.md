---
title: Set up single sign-on
sidebar:
  order: 8
---

Indelible supports a single OpenID Connect (OIDC) provider for sign-in on the
web, Android, and iOS. This guide covers the shared Indelible settings and two
tested providers: [authentik](#authentik) and [Pocket ID](#pocket-id).

## Before you start

You need public HTTPS URLs for both Indelible and your identity provider. The
identity provider must be reachable from users' browsers and phones, while its
OIDC discovery and token endpoints must also be reachable from the `ind-api`
container.

Use this exact callback URL in your identity provider, replacing the hostname
with your Indelible URL:

```text
https://indelible.example.com/api/v1/auth/oauth/oidc/callback
```

Create a **confidential** client that uses the authorization code flow, supports
PKCE with SHA-256, and returns the `openid`, `email`, and `profile` scopes. You
do not need separate clients or callback URLs for Android and iOS: the apps
start the same server-side OIDC flow, then Indelible returns control to the app.

## Configure Indelible

Add the following to the `.env` file beside your Compose file:

```bash
OIDC_ENABLED=true
OIDC_ISSUER_URL=https://id.example.com
OIDC_CLIENT_ID=paste-client-id
OIDC_CLIENT_SECRET=paste-client-secret
OIDC_PROVIDER_NAME=Company SSO
OIDC_SCOPES=openid,email,profile
OIDC_AUTO_CREATE_USERS=true
```

| Variable | Purpose |
| --- | --- |
| `OIDC_ISSUER_URL` | Exact issuer published by the provider's OIDC discovery document. Do not paste the authorization or token endpoint. |
| `OIDC_CLIENT_ID` / `OIDC_CLIENT_SECRET` | Credentials for the confidential client you create below. Keep the secret out of source control. |
| `OIDC_PROVIDER_NAME` | Label shown in **Continue with …** on the sign-in screen, such as `authentik` or `Pocket ID`. |
| `OIDC_SCOPES` | Leave as `openid,email,profile` unless your provider requires another scope to return the same claims. |
| `OIDC_AUTO_CREATE_USERS` | When `true`, SSO can create a new Indelible account if signups are open. When `false`, only existing accounts with the same verified email can sign in. |

The current published Compose examples pass these values to `ind-api`; other
deployment manifests must do the same. The worker and renderer do not need
them.

## authentik

authentik 2025.10 and newer reports `email_verified: false` in its default email
scope. Indelible rejects an explicitly unverified email, so configure an email
scope mapping that reflects how your authentik installation verifies addresses.

### 1. Create a verified-email scope mapping

In the authentik Admin interface, open **Customization → Property Mappings** and
create a **Scope Mapping**:

- Name: `Indelible verified email`
- Scope name: `email`
- Expression:

```python
return {
    "email": request.user.email,
    "email_verified": request.user.attributes.get("email_verified", False),
}
```

Set the `email_verified` attribute to `true` only for users whose address your
provisioning process has verified. If every authentik account is created by a
trusted administrator with a verified address, the mapping can return
`"email_verified": True` directly. Do not make that assertion for
self-registered or unverified accounts. See authentik's
[email scope guidance](https://docs.goauthentik.io/add-secure-apps/providers/oauth2/#email-scope-verification).

### 2. Create the application and provider

1. Open **Applications → Applications**, choose **New Provider**, and name the
   application `Indelible` with the slug `indelible`.
2. Select **OAuth2/OIDC** as the provider type.
3. Set **Client type** to `Confidential` and keep the authorization code flow.
4. Add the Indelible callback URL as a **Strict** redirect URI.
5. Select the `openid` and `profile` scope mappings plus the
   `Indelible verified email` mapping. Do not also select authentik's default
   email mapping.
6. Select a signing key, choose your normal authentication and authorization
   flows, then create the provider and application.

Copy the client ID and client secret. With the `indelible` application slug and
authentik's per-provider issuer mode, the Indelible values are:

```bash
OIDC_ISSUER_URL=https://auth.example.com/application/o/indelible/
OIDC_CLIENT_ID=paste-authentik-client-id
OIDC_CLIENT_SECRET=paste-authentik-client-secret
OIDC_PROVIDER_NAME=authentik
```

Use the issuer shown by authentik if you chose a different issuer mode or slug.

## Pocket ID

Pocket ID is passkey-first and requires HTTPS for a reliable browser session,
including sign-in from the native apps. Make sure its `APP_URL` is the same
public HTTPS URL users open.

### 1. Create the OIDC client

In Pocket ID, open **Settings → Administration → OIDC Clients** and add a
client:

1. Set **Name** to `Indelible`.
2. Set **Client Launch URL** to your Indelible URL.
3. Add the exact Indelible callback URL under **Callback URLs**.
4. Leave **Public Client** off and enable **PKCE**.
5. Save, then copy the generated client ID and client secret.

New Pocket ID clients initially allow no users. Open the client's
**Allowed User Groups** tab and either select the groups that may use Indelible
or choose **Unrestrict** to allow every Pocket ID user. See Pocket ID's
[allowed-groups documentation](https://pocket-id.org/docs/configuration/allowed-groups/).

Confirm that each user's email is marked verified in Pocket ID before they try
to sign in. Then set:

```bash
OIDC_ISSUER_URL=https://id.example.com
OIDC_CLIENT_ID=paste-pocket-id-client-id
OIDC_CLIENT_SECRET=paste-pocket-id-client-secret
OIDC_PROVIDER_NAME=Pocket ID
```

## Restart and verify

Recreate the API container so it reads the new environment:

```bash
docker compose up -d --force-recreate api
```

Check that Indelible advertises the provider:

```bash
curl -fsS https://indelible.example.com/api/v1/auth/providers
```

The response should contain an enabled provider with the ID `oidc` and your
configured display name. Open a private browser window and complete
**Continue with _provider name_**. Then repeat from Android or iOS; no additional
provider configuration is required for the apps.

On a user's first successful OIDC sign-in, Indelible links to an existing active
account with the same normalized, verified email. Later sign-ins use the stable
OIDC subject. If no matching account exists, account creation requires both
`OIDC_AUTO_CREATE_USERS=true` and open Indelible signups; the first account on
an empty instance is the exception and can always be created.

## Troubleshooting

### The SSO button is missing

Call `/api/v1/auth/providers`. If `oidc` is absent, confirm the `OIDC_*`
variables are set on the `ind-api` container, not merely present in Compose's
substitution `.env`, and recreate the container. `OIDC_ENABLED=true` also
requires the issuer URL, client ID, and client secret.

### The provider rejects the callback

Compare its registered redirect URI character for character with
`IND_BASE_URL` plus `/api/v1/auth/oauth/oidc/callback`. Scheme, hostname, port,
path, and trailing slash behavior all matter; prefer an exact or strict match.

### Sign-in reports an unverified email

Indelible rejects `email_verified: false`. Use the verified-email property
mapping for authentik, or mark the user's email verified in Pocket ID. Do not
work around this by asserting verification for addresses your identity system
has not actually verified.

### Pocket ID says the client is unauthorized

The client has no allowed groups. Assign at least one group or unrestrict the
client.

### Sign-in works on the web but not on a phone

Both the Indelible and identity-provider URLs must resolve from the device and
use a certificate it trusts. `localhost` on a physical phone means the phone,
not the server. Private or self-signed certificate authorities must be installed
on every device; for normal deployments, use publicly trusted HTTPS.

### Discovery or token exchange fails

From the `ind-api` container, check that the issuer's
`/.well-known/openid-configuration` document is reachable and that its issuer
exactly matches `OIDC_ISSUER_URL`. Also check DNS, proxy routing, and the
certificate chain from inside the container.

See the [Configuration reference](/docs/reference/configuration/#single-sign-on)
for the complete variable list.
