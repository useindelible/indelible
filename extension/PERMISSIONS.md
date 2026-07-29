# Extension Permissions

This document justifies every permission the Indelible browser extension
requests, so users and reviewers can verify the extension asks only for what it
needs to save and archive pages.

## API permissions (`permissions`)

| Permission     | Why it is needed                                                                                                                                                                                                                                                                    |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `activeTab`    | Grants temporary access to the page in the **current tab only**, and only after the user invokes the extension (toolbar button, context menu, or keyboard shortcut). This is how the extension reads the page the user chose to save without holding standing access to every site. |
| `scripting`    | Inject the capture/extraction script on demand into the active tab to serialize the page for archival. Injection happens only as part of a user-initiated save.                                                                                                                     |
| `contextMenus` | Add the "Save to Indelible" right-click entry.                                                                                                                                                                                                                                      |
| `storage`      | Persist the user's session token and extension settings (e.g. the self-hosted server URL) locally.                                                                                                                                                                                  |
| `tabs`         | Read the active tab's URL and title to label the saved item and to detect whether the current page is already saved.                                                                                                                                                                |

## Host access — `<all_urls>`

Indelible is a read-it-later tool, so the user can choose to save **any** page
they are viewing. The extension therefore needs to be able to read page content
on any origin — but it does so **on demand, per page, never in the background**:

- Page content is accessed through `activeTab` + `scripting`, which scope access
  to the single tab the user explicitly acts on at the moment they save. The
  extension has no persistent content script that runs on every site.
- `<all_urls>` appears in `web_accessible_resources` so the bundled SingleFile
  capture helper scripts (`single-file/*.js`, packaged locally — no remote code)
  can be loaded into the page's frames during a capture the user initiated.
- The full-page archival script (`full-archive.content.ts`) is injected into the
  active tab only when a save is triggered, and is used to serialize the DOM the
  user is looking at.

The extension does **not** read, monitor, or transmit browsing activity on pages
the user has not chosen to save, and it ships no remotely-hosted code.

## Firefox data-collection disclosure

For Firefox (MV3 `browser_specific_settings.data_collection_permissions`) the
extension declares the data it handles during a save:

| Declared                               | Meaning for Indelible                                                                                    |
| -------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `authenticationInfo`                   | The session token used to authenticate saves to the user's Indelible instance, stored locally.           |
| `websiteContent`                       | The page content captured when the user saves a page.                                                    |
| `browsingActivity` / `websiteActivity` | The URL/title of a page at the moment the user saves it, used to create and de-duplicate the saved item. |

All captured data is sent only to the Indelible server the user configured
(the hosted service or their self-hosted instance) and only for pages the user
explicitly saved.
