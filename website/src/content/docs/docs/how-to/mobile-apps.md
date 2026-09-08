---
title: Use the mobile apps
sidebar:
  order: 7
---

Indelible has native apps for iOS and Android with the reader, your library,
highlights, and sync.

## Get the app

Android: install
[Indelible on Google Play](https://play.google.com/store/apps/details?id=com.useindelible.app).

iOS: the App Store release is pending. Until then, run the iOS app through Xcode
via the `iosApp` project in the repository's `mobile/` folder.

You can also build the Android app from source in the same folder: open it in
Android Studio and run `:composeApp:assembleDebug`.

## Connect to your server

On first launch the app asks for your server address. Enter the same URL you open
in your browser, for example `https://indelible.example.com`. If you leave off the
scheme, `https://` is assumed.

The app checks the address by calling your instance's health endpoint before
saving it, so a typo or an offline server is caught immediately.

To switch servers later, tap **Change** next to the server name on the sign-in
screen.

## Unencrypted servers (http)

HTTPS is the default and what we recommend. See
[Security & production checklist](/docs/self-hosting/security/) for terminating
TLS at a reverse proxy.

If your instance only runs on your local network (for example
`http://192.168.1.40:38473`), the app warns you that traffic, including your
password and session, is unencrypted, and asks you to confirm before
continuing. Only do this on a network you trust.

If you want encryption without a public domain, overlay networks like Tailscale
can provide valid HTTPS certificates for private hosts.

## Troubleshooting

- **"Can't reach this server"**: confirm the instance is running and that the
  URL is reachable from your phone's network (a VPN or LAN-only setup on your
  computer isn't automatically reachable from the phone).
- **Sign-in works in the browser but not the app**: make sure you entered the
  API origin, not a proxy path; the app talks to the same origin the web app uses.
