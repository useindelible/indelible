export function normalizeVaultPath(path: string): string {
  return path
    .replace(/\\/g, "/")
    .replace(/\/+/g, "/")
    .replace(/^\/+/, "")
    .replace(/\/+$/, "");
}

export function joinMarkdown(parts: string[]): string {
  return parts
    .map((part) => part.trim())
    .filter((part) => part.length > 0)
    .join("\n\n");
}

export function rewriteBaseFolder(serverPath: string, baseFolder: string): string {
  const normalizedBase = normalizeVaultPath(baseFolder.trim() || "Indelible");
  const normalizedServerPath = normalizeVaultPath(serverPath);
  const parts = normalizedServerPath.split("/");
  if (parts[0] === "Indelible") {
    return normalizeVaultPath([normalizedBase, ...parts.slice(1)].join("/"));
  }
  return normalizeVaultPath([normalizedBase, normalizedServerPath].join("/"));
}

export interface ArtifactPathEntry {
  file_path: string;
  full_document_text_path?: string | null;
}

export function findDuplicateVaultPaths(
  entries: ArtifactPathEntry[],
  baseFolder: string
): string[] {
  const seen = new Set<string>();
  const duplicates = new Set<string>();

  for (const entry of entries) {
    for (const path of artifactEntryPaths(entry, baseFolder)) {
      if (seen.has(path)) {
        duplicates.add(path);
      } else {
        seen.add(path);
      }
    }
  }

  return [...duplicates].sort();
}

function artifactEntryPaths(entry: ArtifactPathEntry, baseFolder: string): string[] {
  const paths = [rewriteBaseFolder(entry.file_path, baseFolder)];
  if (entry.full_document_text_path) {
    paths.push(rewriteBaseFolder(entry.full_document_text_path, baseFolder));
  }
  return paths;
}
