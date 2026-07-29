import { describe, expect, it } from "vitest";
import { zipSync, strToU8 } from "fflate";

import { findDuplicateVaultPaths, joinMarkdown, rewriteBaseFolder } from "./path-utils";
import {
	countWriteOutcomes,
	createRunRequest,
	explicitReimportQueue,
	forceSubjectIds,
	intervalMsForFrequency,
	moveSubjectFileMapEntry,
	nextDeletedReimportQueue,
	parentFolderDeleted,
	parseArtifactManifestZip,
	planFullDocumentCompanionWrite,
	planNoteContentWrite,
	reconcileManifestWrites,
	resolveSecretToken,
	shouldAppendSyncNotification,
	syncOnOpenDelayMs,
	type ReconciliationArtifactEntry,
	type WriteStatusSubject
} from "./sync-utils";

describe("rewriteBaseFolder", () => {
  it("replaces the server-owned Indelible folder with the configured vault folder", () => {
    expect(rewriteBaseFolder("Indelible/books/My Book.md", "Library/Indelible")).toBe(
      "Library/Indelible/books/My Book.md"
    );
  });

  it("prefixes unexpected artifact paths with the configured vault folder", () => {
    expect(rewriteBaseFolder("books/My Book.md", "Exports")).toBe("Exports/books/My Book.md");
  });
});

describe("joinMarkdown", () => {
  it("adds exactly one blank line between non-empty markdown chunks", () => {
    expect(joinMarkdown([" first ", "", "second\n"])).toBe("first\n\nsecond");
  });
});

describe("findDuplicateVaultPaths", () => {
  it("detects duplicate note and full-document paths after base folder rewriting", () => {
    expect(
      findDuplicateVaultPaths(
        [
          {
            file_path: "Indelible/articles/Just a moment.md",
            full_document_text_path: "Indelible/articles/Just a moment Full Text.md"
          },
          {
            file_path: "Indelible/articles/Just a moment.md",
            full_document_text_path: "Indelible/articles/Just a moment Full Text.md"
          }
        ],
        "Exports"
      )
    ).toEqual([
      "Exports/articles/Just a moment Full Text.md",
      "Exports/articles/Just a moment.md"
    ]);
  });

  it("treats normal-note and full-document path collisions as duplicates", () => {
    expect(
      findDuplicateVaultPaths(
        [
          {
            file_path: "Indelible/articles/A Full Text.md"
          },
          {
            file_path: "Indelible/articles/A.md",
            full_document_text_path: "Indelible/articles/A Full Text.md"
          }
        ],
        "Indelible"
      )
    ).toEqual(["Indelible/articles/A Full Text.md"]);
  });
});

describe("countWriteOutcomes", () => {
  it("counts successful and failed writes", () => {
    expect(
      countWriteOutcomes([
        { status: "success" },
        { status: "failed" },
        { status: "success" }
      ])
    ).toEqual({ successful: 2, failed: 1 });
  });
});

describe("resolveSecretToken", () => {
  it("reads and trims a token from SecretStorage", async () => {
    await expect(
      resolveSecretToken("indelible-token", {
        getSecret: (id) => (id === "indelible-token" ? " ind_pat_123 " : null)
      })
    ).resolves.toBe("ind_pat_123");
  });

  it("fails when SecretStorage is unavailable or empty", async () => {
    await expect(resolveSecretToken("", { getSecret: () => "token" })).rejects.toThrow(
      "Choose an Indelible token"
    );
    await expect(resolveSecretToken("indelible-token", undefined)).rejects.toThrow(
      "SecretStorage"
    );
    await expect(
      resolveSecretToken("indelible-token", { getSecret: () => "   " })
    ).rejects.toThrow("empty");
  });
});

describe("sync scheduling helpers", () => {
  it("maps manual and recurring sync frequencies to intervals", () => {
    expect(intervalMsForFrequency("manual")).toBeNull();
    expect(intervalMsForFrequency("1h")).toBe(60 * 60 * 1000);
    expect(intervalMsForFrequency("12h")).toBe(12 * 60 * 60 * 1000);
    expect(intervalMsForFrequency("24h")).toBe(24 * 60 * 60 * 1000);
    expect(intervalMsForFrequency("week")).toBe(7 * 24 * 60 * 60 * 1000);
  });

  it("schedules sync-on-open only when enabled", () => {
    expect(syncOnOpenDelayMs(true)).toBe(2500);
    expect(syncOnOpenDelayMs(false)).toBeNull();
  });
});

describe("forceSubjectIds", () => {
  it("combines failed and deleted queues once for retry/manual sync", () => {
    expect(
      forceSubjectIds(
        {
          lib_failed: { subjectId: "lib_failed" },
          lib_overlap: { subjectId: "lib_overlap" }
        },
        ["lib_deleted", "lib_overlap", ""]
      )
    ).toEqual(["lib_failed", "lib_overlap", "lib_deleted"]);
  });
});

describe("createRunRequest", () => {
  it("builds manual sync requests with forced failed/deleted subjects", () => {
    expect(
      createRunRequest({
        auto: false,
        baseFolder: "Indelible",
        fileToSubjectMap: { "Indelible/articles/A.md": {} },
        failedSubjects: { lib_failed: { subjectId: "lib_failed" } },
        deletedReimportQueue: ["lib_deleted"],
        resyncDeletedFiles: true,
        pathExists: () => true
      })
    ).toEqual({
      auto: false,
      parent_folder_deleted: false,
      force_subject_ids: ["lib_failed", "lib_deleted"]
    });
  });

  it("marks parent folder deleted only for resync-enabled generated vaults", () => {
    expect(
      createRunRequest({
        auto: true,
        baseFolder: "Indelible",
        fileToSubjectMap: { "Indelible/articles/A.md": {} },
        failedSubjects: {},
        deletedReimportQueue: [],
        resyncDeletedFiles: true,
        pathExists: () => false
      })
    ).toMatchObject({ auto: true, parent_folder_deleted: true, force_subject_ids: [] });
  });
});

describe("parentFolderDeleted", () => {
  it("only treats the parent folder as deleted when resync is enabled and generated files exist", () => {
    expect(
      parentFolderDeleted({
        baseFolder: "Indelible",
        fileToSubjectMap: { "Indelible/articles/A.md": {} },
        resyncDeletedFiles: true,
        pathExists: () => false
      })
    ).toBe(true);
    expect(
      parentFolderDeleted({
        baseFolder: "Indelible",
        fileToSubjectMap: { "Indelible/articles/A.md": {} },
        resyncDeletedFiles: false,
        pathExists: () => false
      })
    ).toBe(false);
    expect(
      parentFolderDeleted({
        baseFolder: "Indelible",
        fileToSubjectMap: {},
        resyncDeletedFiles: true,
        pathExists: () => false
      })
    ).toBe(false);
  });
});

describe("shouldAppendSyncNotification", () => {
  it("allows notification-only manifests", () => {
    expect(shouldAppendSyncNotification(0, [])).toBe(true);
  });

  it("allows notifications when every artifact entry wrote successfully", () => {
    expect(
      shouldAppendSyncNotification(2, [{ status: "success" }, { status: "success" }])
    ).toBe(true);
  });

  it("blocks notifications when every write failed", () => {
    expect(shouldAppendSyncNotification(2, [{ status: "failed" }, { status: "failed" }])).toBe(
      false
    );
  });

  it("blocks notifications when only some writes succeeded", () => {
    expect(
      shouldAppendSyncNotification(2, [{ status: "success" }, { status: "failed" }])
    ).toBe(false);
  });
});

describe("parseArtifactManifestZip", () => {
  it("parses artifact.json from a downloaded zip", () => {
    const manifest = {
      version: 1,
      run_id: "run_1",
      generated_at: "2026-05-08T00:00:00Z",
      entries: [{ subject_id: "lib_1" }]
    };
    const zipped = zipSync({
      "artifact.json": strToU8(JSON.stringify(manifest))
    });

    expect(parseArtifactManifestZip("art_1", zipped.buffer as ArrayBuffer)).toEqual(manifest);
  });

  it("rejects missing or invalid artifact.json", () => {
    expect(() => parseArtifactManifestZip("art_1", zipSync({}).buffer as ArrayBuffer)).toThrow(
      "did not contain artifact.json"
    );
    const zipped = zipSync({ "artifact.json": strToU8(JSON.stringify({ version: 1 })) });
    expect(() => parseArtifactManifestZip("art_1", zipped.buffer as ArrayBuffer)).toThrow(
      "invalid artifact.json"
    );
  });
});

describe("nextDeletedReimportQueue", () => {
  it("does not queue deleted files when resync deleted files is disabled", () => {
    expect(nextDeletedReimportQueue(["lib_existing"], "lib_deleted", false)).toEqual([
      "lib_existing"
    ]);
  });

  it("queues deleted files once when resync deleted files is enabled", () => {
    expect(nextDeletedReimportQueue(["lib_existing"], "lib_deleted", true)).toEqual([
      "lib_existing",
      "lib_deleted"
    ]);
    expect(nextDeletedReimportQueue(["lib_deleted"], "lib_deleted", true)).toEqual([
      "lib_deleted"
    ]);
  });
});

describe("explicitReimportQueue", () => {
  it("queues explicit current-document reimports once", () => {
    expect(explicitReimportQueue(["lib_existing"], "lib_deleted")).toEqual([
      "lib_existing",
      "lib_deleted"
    ]);
    expect(explicitReimportQueue(["lib_deleted"], "lib_deleted")).toEqual(["lib_deleted"]);
  });
});

describe("moveSubjectFileMapEntry", () => {
  it("moves tracked file map entries without changing the server path", () => {
    const moved = moveSubjectFileMapEntry(
      {
        "Indelible/articles/A.md": {
          subjectId: "lib_1",
          localPath: "Indelible/articles/A.md",
          serverPath: "Indelible/articles/A.md",
          updatedAt: "old"
        }
      },
      "Indelible/articles/A.md",
      "Notes/A moved.md",
      "2026-05-08T00:00:00.000Z"
    );

    expect(moved["Indelible/articles/A.md"]).toBeUndefined();
    expect(moved["Notes/A moved.md"]).toMatchObject({
      subjectId: "lib_1",
      localPath: "Notes/A moved.md",
      serverPath: "Indelible/articles/A.md",
      updatedAt: "2026-05-08T00:00:00.000Z"
    });
  });
});

describe("reconcileManifestWrites", () => {
  it("passes when successful entries have map entries and written files", () => {
    const result = reconcileManifestWrites({
      entries: [
        {
          subject_id: "lib_1",
          file_path: "Indelible/articles/A.md",
          full_content: "# A",
          full_document_text_path: "Indelible/articles/A Full Text.md",
          full_document_text: "# A\n\nBody"
        }
      ],
      ackSubjects: [{ subject_id: "lib_1", status: "success" }],
      baseFolder: "Exports",
      fileToSubjectMap: {
        "Exports/articles/A.md": { subjectId: "lib_1" }
      },
      fileExists: (path) =>
        path === "Exports/articles/A.md" || path === "Exports/articles/A Full Text.md"
    });

    expect(result).toMatchObject({
      ok: true,
      artifactEntryCount: 1,
      successfulEntryCount: 1,
      uniqueExpectedPathCount: 2,
      writtenVaultFileCount: 2,
      pluginMapMatchCount: 1
    });
  });

  it("fails when a successful write is missing from the vault", () => {
    const result = reconcileManifestWrites({
      entries: [
        {
          subject_id: "lib_1",
          file_path: "Indelible/articles/A.md",
          full_content: "# A"
        }
      ],
      ackSubjects: [{ subject_id: "lib_1", status: "success" }],
      baseFolder: "Indelible",
      fileToSubjectMap: {
        "Indelible/articles/A.md": { subjectId: "lib_1" }
      },
      fileExists: () => false
    });

    expect(result.ok).toBe(false);
    expect(result.error).toContain("missing files: Indelible/articles/A.md");
  });

  it("fails when a successful write is missing from the plugin map", () => {
    const result = reconcileManifestWrites({
      entries: [
        {
          subject_id: "lib_1",
          file_path: "Indelible/articles/A.md",
          full_content: "# A"
        }
      ],
      ackSubjects: [{ subject_id: "lib_1", status: "success" }],
      baseFolder: "Indelible",
      fileToSubjectMap: {},
      fileExists: () => true
    });

    expect(result.ok).toBe(false);
    expect(result.error).toContain("missing map entries: Indelible/articles/A.md");
  });
});

describe("planNoteContentWrite", () => {
  it("creates a generated note when append-only content arrives after a rename or move", () => {
    const appendOnlyContent = "## New highlights added May 5, 2026 at 09:05\n\n- New highlight";

    const plan = planNoteContentWrite({
      path: "Indelible/articles/Example.md",
      appendOnlyContent
    });

    expect(plan).toEqual({
      action: "create",
      content: appendOnlyContent,
      ackContentHash: true
    });
  });

  it("preserves local edits when appending new highlights", () => {
    const serverContent = "## Metadata\n\n- Full Title: Example\n\n## Highlights\n\n- Old highlight";
    const localContent = `${serverContent}\n\nMy local note`;
    const appendOnlyContent = "## New highlights added May 5, 2026 at 09:05\n\n- New highlight";

    const plan = planNoteContentWrite({
      path: "Indelible/articles/Example.md",
      existingContent: localContent,
      existingHash: "local-edited-hash",
      mappedLastContentHash: "last-server-hash",
      appendOnlyContent
    });

    expect(plan).toEqual({
      action: "append",
      content: joinMarkdown([localContent, appendOnlyContent]),
      ackContentHash: false
    });
  });

  it("refuses to overwrite locally edited notes during full rebuilds", () => {
    const serverContent = "## Metadata\n\n- Full Title: Example";
    const localContent = `${serverContent}\n\nMy local note`;

    const plan = planNoteContentWrite({
      path: "Indelible/articles/Example.md",
      existingContent: localContent,
      existingHash: "local-edited-hash",
      mappedLastContentHash: "last-server-hash",
      fullContent: "## Metadata\n\n- Full Title: Example rebuilt"
    });

    expect(plan).toEqual({
      action: "error",
      error:
        "Indelible/articles/Example.md has local edits; refusing to overwrite without append-only content.",
      ackContentHash: false
    });
  });
});

describe("planFullDocumentCompanionWrite", () => {
  it("creates full-text companions when no local file exists", () => {
    expect(
      planFullDocumentCompanionWrite({
        fullDocumentTextPath: "Indelible/articles/A Full Text.md",
        fullDocumentText: " Full body "
      })
    ).toEqual({ action: "create", content: "Full body", ackFullDocumentHash: true });
  });

  it("replaces full-text companions only when local hash matches the last exported hash", () => {
    expect(
      planFullDocumentCompanionWrite({
        fullDocumentTextPath: "Indelible/articles/A Full Text.md",
        fullDocumentText: "New full body",
        existingHash: "old-hash",
        mappedLastFullDocumentHash: "old-hash"
      })
    ).toEqual({ action: "replace", content: "New full body", ackFullDocumentHash: true });
    expect(
      planFullDocumentCompanionWrite({
        fullDocumentTextPath: "Indelible/articles/A Full Text.md",
        fullDocumentText: "New full body",
        existingHash: "locally-edited-hash",
        mappedLastFullDocumentHash: "old-hash"
      })
    ).toEqual({ action: "none", ackFullDocumentHash: false });
  });
});

describe("temp-vault artifact application", () => {
  it("writes exact note and companion paths, preserves edits on append, and rebuilds deleted notes", () => {
    const state = createTempVaultState("Vault");
    const firstEntries: ReconciliationArtifactEntry[] = [
      {
        subject_id: "lib_a",
        file_path: "Indelible/articles/Example - subject-a.md",
        full_content:
          "## Metadata\n\n- Full Title: Example\n- Category: #articles\n- Document Tags: [[research]] \n- URL: https://example.com/a\n- Summary: Useful summary.\n\n[Full document text](Indelible/articles/Example%20-%20subject-a%20Full%20Text.md)\n\n## Highlights\n\n- First highlight ([Location 1](https://example.com/a#h1))\n    - Tags: [[quote]] \n    - Note: Remember this",
        full_document_text_path: "Indelible/articles/Example - subject-a Full Text.md",
        full_document_text:
          "Lead paragraph.\n\n| Metric | Value |\n| --- | --- |\n| A | 1 |\n\nSecond paragraph."
      },
      {
        subject_id: "lib_b",
        file_path: "Indelible/articles/Example - subject-b.md",
        full_content:
          "## Metadata\n\n- Full Title: Example\n- Category: #articles\n\n## Highlights\n\n- Other highlight",
        full_document_text_path: "Indelible/articles/Example - subject-b Full Text.md",
        full_document_text: "Companion body for the second same-title subject."
      }
    ];

    const firstAck = applyTempVaultEntries(state, firstEntries);

    expect(firstAck).toEqual([
      {
        subject_id: "lib_a",
        status: "success",
        last_content_hash: tempHash(firstEntries[0].full_content),
        last_full_document_hash: tempHash(firstEntries[0].full_document_text)
      },
      {
        subject_id: "lib_b",
        status: "success",
        last_content_hash: tempHash(firstEntries[1].full_content),
        last_full_document_hash: tempHash(firstEntries[1].full_document_text)
      }
    ]);
    expect([...state.vault.keys()].sort()).toEqual([
      "Vault/articles/Example - subject-a Full Text.md",
      "Vault/articles/Example - subject-a.md",
      "Vault/articles/Example - subject-b Full Text.md",
      "Vault/articles/Example - subject-b.md"
    ]);
    expect(state.vault.get("Vault/articles/Example - subject-a.md")).toBe(firstEntries[0].full_content);
    expect(state.vault.get("Vault/articles/Example - subject-a Full Text.md")).toBe(
      firstEntries[0].full_document_text
    );
    expect(Object.keys(state.fileToSubjectMap).sort()).toEqual([
      "Vault/articles/Example - subject-a.md",
      "Vault/articles/Example - subject-b.md"
    ]);
    expect(
      reconcileManifestWrites({
        entries: firstEntries,
        ackSubjects: firstAck,
        baseFolder: state.baseFolder,
        fileToSubjectMap: state.fileToSubjectMap,
        fileExists: (path) => state.vault.has(path)
      }).ok
    ).toBe(true);

    state.vault.set(
      "Vault/articles/Example - subject-a.md",
      `${state.vault.get("Vault/articles/Example - subject-a.md")}\n\nLocal observation.`
    );
    const appendEntry: ReconciliationArtifactEntry = {
      subject_id: "lib_a",
      file_path: "Indelible/articles/Example - subject-a.md",
      append_only_content:
        "## New highlights added May 5, 2026 at 09:05\n\n- Newly appended highlight"
    };

    const appendAck = applyTempVaultEntries(state, [appendEntry]);

    expect(appendAck).toEqual([{ subject_id: "lib_a", status: "success" }]);
    expect(state.vault.get("Vault/articles/Example - subject-a.md")).toContain("Local observation.");
    expect(state.vault.get("Vault/articles/Example - subject-a.md")).toContain(
      "Newly appended highlight"
    );

    const beforeNoop = new Map(state.vault);
    expect(applyTempVaultEntries(state, [])).toEqual([]);
    expect(state.vault).toEqual(beforeNoop);

    state.vault.delete("Vault/articles/Example - subject-b.md");
    const rebuildEntry: ReconciliationArtifactEntry = {
      subject_id: "lib_b",
      file_path: "Indelible/articles/Example - subject-b.md",
      full_content:
        "## Metadata\n\n- Full Title: Example rebuilt\n- Category: #articles\n\n## Highlights\n\n- Rebuilt highlight"
    };

    const rebuildAck = applyTempVaultEntries(state, [rebuildEntry]);

    expect(rebuildAck).toEqual([
      {
        subject_id: "lib_b",
        status: "success",
        last_content_hash: tempHash(rebuildEntry.full_content)
      }
    ]);
    expect(state.vault.get("Vault/articles/Example - subject-b.md")).toBe(rebuildEntry.full_content);
  });

  it("carries subject_kind into sync state and preserves it across an append ack", () => {
    const state = createTempVaultState("Vault");
    const initialEntry: ReconciliationArtifactEntry = {
      subject_id: "lib_kind",
      subject_kind: "library_entry",
      file_path: "Indelible/articles/Kinded.md",
      full_content: "## Metadata\n\n- Full Title: Kinded\n\n## Highlights\n\n- First highlight"
    };

    applyTempVaultEntries(state, [initialEntry]);

    const mapPath = "Vault/articles/Kinded.md";
    expect(state.fileToSubjectMap[mapPath]?.subjectKind).toBe("library_entry");

    const appendEntry: ReconciliationArtifactEntry = {
      subject_id: "lib_kind",
      file_path: "Indelible/articles/Kinded.md",
      append_only_content: "## New highlights\n\n- Appended highlight"
    };

    const appendAck = applyTempVaultEntries(state, [appendEntry]);

    expect(appendAck).toEqual([
      expect.objectContaining({ subject_id: "lib_kind", status: "success" })
    ]);
    expect(state.fileToSubjectMap[mapPath]?.subjectKind).toBe("library_entry");
  });
});

interface TempVaultMapEntry {
  subjectId: string;
  subjectKind?: string;
  lastContentHash?: string;
  lastFullDocumentHash?: string;
}

interface TempVaultState {
  baseFolder: string;
  vault: Map<string, string>;
  fileToSubjectMap: Record<string, TempVaultMapEntry>;
}

type TempAckSubject = WriteStatusSubject & {
  last_content_hash?: string;
  last_full_document_hash?: string;
};

function createTempVaultState(baseFolder: string): TempVaultState {
  return {
    baseFolder,
    vault: new Map(),
    fileToSubjectMap: {}
  };
}

function applyTempVaultEntries(
  state: TempVaultState,
  entries: ReconciliationArtifactEntry[]
): TempAckSubject[] {
  return entries.map((entry) => applyTempVaultEntry(state, entry));
}

function applyTempVaultEntry(
  state: TempVaultState,
  entry: ReconciliationArtifactEntry
): TempAckSubject {
  const localPath = rewriteBaseFolder(entry.file_path, state.baseFolder);
  const mapped = state.fileToSubjectMap[localPath];
  const existingContent = state.vault.get(localPath);
  const notePlan = planNoteContentWrite({
    path: localPath,
    existingContent,
    existingHash: existingContent ? tempHash(existingContent) : undefined,
    mappedLastContentHash: mapped?.lastContentHash,
    fullContent: entry.full_content ?? undefined,
    appendOnlyContent: entry.append_only_content ?? undefined
  });

  if (notePlan.action === "error") {
    return { subject_id: entry.subject_id, status: "failed", error: notePlan.error };
  }

  let lastContentHash: string | undefined;
  if (
    notePlan.action === "create" ||
    notePlan.action === "replace" ||
    notePlan.action === "append"
  ) {
    state.vault.set(localPath, notePlan.content);
    if (notePlan.ackContentHash) {
      lastContentHash = tempHash(notePlan.content);
    }
  }

  const fullPath = entry.full_document_text_path
    ? rewriteBaseFolder(entry.full_document_text_path, state.baseFolder)
    : undefined;
  const existingFullText = fullPath ? state.vault.get(fullPath) : undefined;
  const fullPlan = planFullDocumentCompanionWrite({
    fullDocumentTextPath: fullPath,
    fullDocumentText: entry.full_document_text,
    existingHash: existingFullText ? tempHash(existingFullText) : undefined,
    mappedLastFullDocumentHash: mapped?.lastFullDocumentHash
  });

  let lastFullDocumentHash: string | undefined;
  if ((fullPlan.action === "create" || fullPlan.action === "replace") && fullPath) {
    state.vault.set(fullPath, fullPlan.content);
    lastFullDocumentHash = tempHash(fullPlan.content);
  }

  state.fileToSubjectMap[localPath] = {
    subjectId: entry.subject_id,
    subjectKind: entry.subject_kind ?? mapped?.subjectKind,
    lastContentHash: lastContentHash ?? mapped?.lastContentHash,
    lastFullDocumentHash: lastFullDocumentHash ?? mapped?.lastFullDocumentHash
  };

  return {
    subject_id: entry.subject_id,
    status: "success",
    ...(lastContentHash ? { last_content_hash: lastContentHash } : {}),
    ...(lastFullDocumentHash ? { last_full_document_hash: lastFullDocumentHash } : {})
  };
}

function tempHash(content: string | null | undefined): string {
  return `hash:${content ?? ""}`;
}
