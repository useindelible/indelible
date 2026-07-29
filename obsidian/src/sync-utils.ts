import { strFromU8, unzipSync } from "fflate";

import { joinMarkdown, rewriteBaseFolder } from "./path-utils";

export type SyncFrequency = "manual" | "1h" | "12h" | "24h" | "week";

export interface FailedSubjectLike {
  subjectId: string;
}

export interface SubjectFileMapEntryLike {
  subjectId: string;
  subjectKind?: string;
  localPath: string;
  updatedAt: string;
}

export interface SecretStorageLike {
  getSecret(id: string): string | null | Promise<string | null>;
}

export interface ArtifactManifestLike {
  version: number;
  run_id: string;
  generated_at: string;
  entries: unknown[];
}

export interface CreateRunRequest {
  parent_folder_deleted: boolean;
  auto: boolean;
  force_subject_ids: string[];
}

export const SYNC_INTERVALS_MS: Record<SyncFrequency, number | null> = {
  manual: null,
  "1h": 60 * 60 * 1000,
  "12h": 12 * 60 * 60 * 1000,
  "24h": 24 * 60 * 60 * 1000,
  week: 7 * 24 * 60 * 60 * 1000
};

export async function resolveSecretToken(
  tokenSecretName: string,
  secretStorage: SecretStorageLike | undefined
): Promise<string> {
  if (!tokenSecretName.trim()) {
    throw new Error("Choose an Indelible token from Obsidian SecretStorage.");
  }
  if (!secretStorage) {
    throw new Error("Obsidian 1.11.4 or newer is required for SecretStorage.");
  }
  const token = await Promise.resolve(secretStorage.getSecret(tokenSecretName));
  if (!token?.trim()) {
    throw new Error("The selected SecretStorage entry is empty.");
  }
  return token.trim();
}

export function intervalMsForFrequency(frequency: SyncFrequency): number | null {
  return SYNC_INTERVALS_MS[frequency];
}

export function syncOnOpenDelayMs(syncOnOpen: boolean): number | null {
  return syncOnOpen ? 2500 : null;
}

export function forceSubjectIds(
  failedSubjects: Record<string, FailedSubjectLike | undefined>,
  deletedReimportQueue: string[]
): string[] {
  return unique([...Object.keys(failedSubjects), ...deletedReimportQueue]);
}

export function parentFolderDeleted(input: {
  baseFolder: string;
  fileToSubjectMap: Record<string, unknown>;
  resyncDeletedFiles: boolean;
  pathExists: (path: string) => boolean;
}): boolean {
  const baseFolder = input.baseFolder.trim() || "Indelible";
  return (
    input.resyncDeletedFiles &&
    Object.keys(input.fileToSubjectMap).length > 0 &&
    !input.pathExists(baseFolder)
  );
}

export function createRunRequest(input: {
  auto: boolean;
  baseFolder: string;
  fileToSubjectMap: Record<string, unknown>;
  failedSubjects: Record<string, FailedSubjectLike | undefined>;
  deletedReimportQueue: string[];
  resyncDeletedFiles: boolean;
  pathExists: (path: string) => boolean;
}): CreateRunRequest {
  return {
    parent_folder_deleted: parentFolderDeleted(input),
    auto: input.auto,
    force_subject_ids: forceSubjectIds(input.failedSubjects, input.deletedReimportQueue)
  };
}

export function moveSubjectFileMapEntry<T extends SubjectFileMapEntryLike>(
  fileToSubjectMap: Record<string, T>,
  oldPath: string,
  newPath: string,
  nowIso: string
): Record<string, T> {
  const entry = fileToSubjectMap[oldPath];
  if (!entry) {
    return fileToSubjectMap;
  }
  const next = { ...fileToSubjectMap };
  delete next[oldPath];
  next[newPath] = {
    ...entry,
    localPath: newPath,
    updatedAt: nowIso
  };
  return next;
}

export function explicitReimportQueue(currentQueue: string[], subjectId: string): string[] {
  return unique([...currentQueue, subjectId]);
}

export function parseArtifactManifestZip(
  artifactId: string,
  arrayBuffer: ArrayBuffer
): ArtifactManifestLike {
  const files = unzipSync(new Uint8Array(arrayBuffer));
  const artifactJson = files["artifact.json"];
  if (!artifactJson) {
    throw new Error(`Artifact ${artifactId} did not contain artifact.json.`);
  }
  const parsed = JSON.parse(strFromU8(artifactJson)) as Partial<ArtifactManifestLike>;
  if (
    typeof parsed.version !== "number" ||
    typeof parsed.run_id !== "string" ||
    typeof parsed.generated_at !== "string" ||
    !Array.isArray(parsed.entries)
  ) {
    throw new Error(`Artifact ${artifactId} contained invalid artifact.json.`);
  }
  return parsed as ArtifactManifestLike;
}

export interface WriteStatusSubject {
  subject_id?: string;
  status: "success" | "failed";
  error?: string;
}

export interface WriteCounts {
  successful: number;
  failed: number;
}

export function countWriteOutcomes(subjects: WriteStatusSubject[]): WriteCounts {
  return subjects.reduce<WriteCounts>(
    (counts, subject) => {
      if (subject.status === "success") {
        counts.successful += 1;
      } else {
        counts.failed += 1;
      }
      return counts;
    },
    { successful: 0, failed: 0 }
  );
}

export function shouldAppendSyncNotification(
  entryCount: number,
  ackSubjects: WriteStatusSubject[]
): boolean {
  if (entryCount === 0) {
    return true;
  }
  const counts = countWriteOutcomes(ackSubjects);
  return counts.failed === 0 && counts.successful === entryCount;
}

export function nextDeletedReimportQueue(
  currentQueue: string[],
  subjectId: string,
  resyncDeletedFiles: boolean
): string[] {
  if (!resyncDeletedFiles || currentQueue.includes(subjectId)) {
    return currentQueue;
  }
  return [...currentQueue, subjectId];
}

export interface NoteContentWriteInput {
  path: string;
  existingContent?: string;
  existingHash?: string;
  mappedLastContentHash?: string;
  fullContent?: string;
  appendOnlyContent?: string;
}

export type NoteContentWritePlan =
  | { action: "none"; ackContentHash: false }
  | { action: "create" | "replace" | "append"; content: string; ackContentHash: boolean }
  | { action: "error"; error: string; ackContentHash: false };

export function planNoteContentWrite(input: NoteContentWriteInput): NoteContentWritePlan {
  const fullContent = trimToContent(input.fullContent);
  const appendOnlyContent = trimToContent(input.appendOnlyContent);
  const hasExisting = input.existingContent !== undefined;

  if (!hasExisting) {
    if (fullContent) {
      return { action: "create", content: fullContent, ackContentHash: true };
    }
    if (appendOnlyContent) {
      return { action: "create", content: appendOnlyContent, ackContentHash: true };
    }
    return { action: "none", ackContentHash: false };
  }
  const existingContent = input.existingContent ?? "";

  if (fullContent) {
    if (
      input.mappedLastContentHash &&
      input.existingHash &&
      input.existingHash === input.mappedLastContentHash
    ) {
      return { action: "replace", content: fullContent, ackContentHash: true };
    }
    if (appendOnlyContent) {
      return {
        action: "append",
        content: joinMarkdown([existingContent, appendOnlyContent]),
        ackContentHash: false
      };
    }
    return {
      action: "error",
      error: `${input.path} has local edits; refusing to overwrite without append-only content.`,
      ackContentHash: false
    };
  }

  if (appendOnlyContent) {
    const canAckHash =
      Boolean(input.mappedLastContentHash) &&
      Boolean(input.existingHash) &&
      input.existingHash === input.mappedLastContentHash;
    return {
      action: "append",
      content: joinMarkdown([existingContent, appendOnlyContent]),
      ackContentHash: canAckHash
    };
  }

  return { action: "none", ackContentHash: false };
}

export interface FullDocumentCompanionWriteInput {
  fullDocumentTextPath?: string | null;
  fullDocumentText?: string | null;
  existingHash?: string;
  mappedLastFullDocumentHash?: string;
}

export type FullDocumentCompanionWritePlan =
  | { action: "none"; ackFullDocumentHash: false }
  | { action: "create" | "replace"; content: string; ackFullDocumentHash: true };

export function planFullDocumentCompanionWrite(
  input: FullDocumentCompanionWriteInput
): FullDocumentCompanionWritePlan {
  const content = trimToContent(input.fullDocumentText);
  if (!input.fullDocumentTextPath || !content) {
    return { action: "none", ackFullDocumentHash: false };
  }
  if (!input.existingHash) {
    return { action: "create", content, ackFullDocumentHash: true };
  }
  if (
    input.mappedLastFullDocumentHash &&
    input.existingHash === input.mappedLastFullDocumentHash
  ) {
    return { action: "replace", content, ackFullDocumentHash: true };
  }
  return { action: "none", ackFullDocumentHash: false };
}

export interface ReconciliationArtifactEntry {
  subject_id: string;
  subject_kind?: string;
  file_path: string;
  full_content?: string | null;
  append_only_content?: string | null;
  full_document_text_path?: string | null;
  full_document_text?: string | null;
}

export interface ReconciliationSubjectFileMapEntry {
  subjectId: string;
  subjectKind?: string;
}

export interface ManifestReconciliationInput {
  entries: ReconciliationArtifactEntry[];
  ackSubjects: WriteStatusSubject[];
  baseFolder: string;
  fileToSubjectMap: Record<string, ReconciliationSubjectFileMapEntry | undefined>;
  fileExists: (path: string) => boolean;
}

export interface ManifestReconciliationResult {
  ok: boolean;
  artifactEntryCount: number;
  successfulEntryCount: number;
  uniqueExpectedPathCount: number;
  writtenVaultFileCount: number;
  pluginMapMatchCount: number;
  error?: string;
}

export function reconcileManifestWrites(
  input: ManifestReconciliationInput
): ManifestReconciliationResult {
  const successfulSubjectIds = new Set(
    input.ackSubjects
      .filter((subject) => subject.status === "success" && subject.subject_id)
      .map((subject) => subject.subject_id as string)
  );
  const successfulEntries = input.entries.filter((entry) => successfulSubjectIds.has(entry.subject_id));
  const expectedPaths = new Set<string>();
  const missingMapPaths: string[] = [];

  for (const entry of successfulEntries) {
    const localPath = rewriteBaseFolder(entry.file_path, input.baseFolder);
    if (input.fileToSubjectMap[localPath]?.subjectId === entry.subject_id) {
      // Count every successfully mapped subject, even if this sync only wrote its companion.
    } else {
      missingMapPaths.push(localPath);
    }

    if (hasText(entry.full_content) || hasText(entry.append_only_content)) {
      expectedPaths.add(localPath);
    }
    if (entry.full_document_text_path && hasText(entry.full_document_text)) {
      expectedPaths.add(rewriteBaseFolder(entry.full_document_text_path, input.baseFolder));
    }
  }

  const expectedPathList = [...expectedPaths].sort();
  const missingVaultPaths = expectedPathList.filter((path) => !input.fileExists(path));
  const pluginMapMatchCount = successfulEntries.length - missingMapPaths.length;
  const writtenVaultFileCount = expectedPathList.length - missingVaultPaths.length;
  const result: ManifestReconciliationResult = {
    ok: missingMapPaths.length === 0 && missingVaultPaths.length === 0,
    artifactEntryCount: input.entries.length,
    successfulEntryCount: successfulEntries.length,
    uniqueExpectedPathCount: expectedPathList.length,
    writtenVaultFileCount,
    pluginMapMatchCount
  };

  if (!result.ok) {
    result.error = [
      `Obsidian write reconciliation failed for ${successfulEntries.length} successful entries`,
      `expected ${expectedPathList.length} written vault files, found ${writtenVaultFileCount}`,
      `plugin map matched ${pluginMapMatchCount} entries`,
      missingVaultPaths.length > 0
        ? `missing files: ${missingVaultPaths.slice(0, 5).join(", ")}`
        : null,
      missingMapPaths.length > 0
        ? `missing map entries: ${missingMapPaths.slice(0, 5).join(", ")}`
        : null
    ]
      .filter((part): part is string => part !== null)
      .join("; ");
  }

  return result;
}

function hasText(value: string | null | undefined): boolean {
  return value !== null && value !== undefined && value.trim().length > 0;
}

function trimToContent(value: string | null | undefined): string | undefined {
  let trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function unique(values: string[]): string[] {
  return Array.from(new Set(values.filter((value) => value.trim().length > 0)));
}
