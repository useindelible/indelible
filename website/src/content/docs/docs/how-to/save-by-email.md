---
title: Save content by email
sidebar:
  order: 1
---

Every Indelible account has two personal email addresses:

- **Feed address**: `<token>@feed.useindelible.com`. Subscribe newsletters with
  this address and each issue lands in your Feed.
- **Library address**: `<token>@library.useindelible.com`. Forward a one-off
  email (or email a link) and it's archived straight into your Library.

Find both addresses in **Settings**.

Self-hosted instances must set `EMAIL_FEED_DOMAIN` and `EMAIL_LIBRARY_DOMAIN` to their
own domains to use this feature. Both are unset by default. With them unset, accounts
have no email address at all (there is no shared-domain fallback) and the addresses do
not appear in Settings.
