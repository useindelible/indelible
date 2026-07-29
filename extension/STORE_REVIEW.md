# Store review notes

## Browser-managed sign-in

Indelible uses the Manifest V3 `identity` permission only for
`identity.getRedirectURL('indelible')` and
`identity.launchWebAuthFlow({ interactive: true })`.

The extension generates a random OAuth state value and an S256 PKCE verifier.
It stores the pending verifier locally before opening the Indelible server's
`/api/v1/auth/extension/start` endpoint. The server redirects to its own web
consent screen. After consent, the browser intercepts the registered callback,
closes the auth window, and returns the callback URL to the extension. The
extension validates the callback and state, then exchanges the one-time code
with the original verifier. Passwords are entered only into the user's chosen
Indelible server and are never visible to the extension.

The extension does not use `tabs.onUpdated` for authentication and does not
request host permissions.

## Stable store identities

The Chrome Web Store item ID is `lblngpkieoichinegfhgacmcjbahjbek`. Chrome
builds carry that item's public `key` in `manifest.json`, so store and unpacked
builds share this callback:

```text
https://lblngpkieoichinegfhgacmcjbahjbek.chromiumapp.org/indelible
```

The public key is safe to distribute; the store signing key remains private.
Edge must use its own reserved store identity and callback instead of
inheriting Chrome's key.

Firefox has the fixed Gecko ID `extension@useindelible.com`; its deterministic
callback is:

```text
https://38bd18db5de5caccb6ab6c1271fec03ec1662d5c.extensions.allizom.org/indelible
```

## Forks and local unpacked builds

Forks must set their own stable extension IDs and replace
`EXTENSION_REDIRECT_URIS` with their exact callbacks. For unpacked Edge or
private builds whose ID differs from the store build, call
`identity.getRedirectURL('indelible')` in that build and add the returned URL
only to the local server configuration. Development callbacks should not be
added to hosted production defaults.
