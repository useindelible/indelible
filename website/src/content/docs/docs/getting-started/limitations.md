---
title: What to expect
sidebar:
  order: 3
---

Indelible aims to be honest about what it does today. Here's the current scope so
there are no surprises.

## Capture

- **Articles, newsletters, PDFs, and EPUBs are archived in full.** The text and
  images are fetched and stored at save time, so your copy survives even if the
  original disappears.
- **Videos are saved with their transcript and metadata, not the media file.**
  For a YouTube or other video page, Indelible captures the transcript (where one is
  available) and details like title and channel; it does not download or host the
  video itself.
- **Tweets and podcasts** are captured as their text/show-notes and enclosure
  metadata, not as archived media.
- **Podcast transcripts from Spotify and Apple Podcasts are not supported yet.**
  Indelible does not extract transcripts from Spotify or Apple Podcasts at this time.

## Hosting

- **Indelible Cloud is coming soon.** Today, Indelible is self-hosted; run it with
  Docker on your own hardware. See [Install with Docker](/docs/self-hosting/install/).
- **The web interface is served separately** from the backend images. It's a static
  single-page app you host alongside your API. A dedicated web-hosting guide is
  coming.

## Operational notes

- **Rate limiting is per-instance.** Single-node deployments are fully supported;
  coordinated rate limiting across multiple API replicas is not wired up yet.
- **Inbound email needs a provider.** Saving by email requires configuring an email
  provider (Resend) and the matching DNS, not just an address.
- **Mila and semantic search need an AI provider.** Full-text search works out of the
  box; semantic search and the Mila assistant require configuring an embeddings/chat
  provider.
- **Reading EPUBs and PDFs on mobile is coming next.** The mobile apps don't open
  EPUB or PDF files yet; that lands in the next iteration.
