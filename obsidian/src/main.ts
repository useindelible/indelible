import {
  App,
  Notice,
  Plugin,
  PluginSettingTab,
  requestUrl,
  SecretComponent,
  Setting,
  TAbstractFile,
  TFile,
	TFolder,
	normalizePath
} from "obsidian";

import { findDuplicateVaultPaths, joinMarkdown, rewriteBaseFolder } from "./path-utils";
import {
	countWriteOutcomes,
	createRunRequest,
	explicitReimportQueue,
	intervalMsForFrequency,
	moveSubjectFileMapEntry,
	nextDeletedReimportQueue,
	parseArtifactManifestZip,
	planFullDocumentCompanionWrite,
	planNoteContentWrite,
	reconcileManifestWrites,
	resolveSecretToken,
	shouldAppendSyncNotification,
	syncOnOpenDelayMs
} from "./sync-utils";
import type { SecretStorageLike, SyncFrequency } from "./sync-utils";

interface SubjectFileMapEntry {
  subjectId: string;
  subjectKind?: string;
  bookId: string;
  serverPath: string;
  localPath: string;
  fullDocumentPath?: string;
  lastContentHash?: string;
  lastFullDocumentHash?: string;
  updatedAt: string;
}

interface FailedSubject {
  subjectId: string;
  error: string;
  lastAttemptAt: string;
}

interface IndelibleSettings {
  tokenSecretName: string;
  apiBaseUrl: string;
  baseFolder: string;
  syncFrequency: SyncFrequency;
  syncOnOpen: boolean;
  resyncDeletedFiles: boolean;
  reimportConfirmation: boolean;
  customizeFormattingUrl: string;
  /** Maximum minutes to wait for a sync run to finish before giving up. */
  pollMaxMinutes: number;
  failedSubjects: Record<string, FailedSubject>;
  deletedReimportQueue: string[];
  fileToSubjectMap: Record<string, SubjectFileMapEntry>;
  lastSyncAt?: string;
  lastSyncStatus?: string;
}

interface ObsidianRunStatusResponse {
  run_id: string;
  task_status: string;
  total_documents: number;
  documents_exported: number;
  is_finished: boolean;
  artifact_ids: string[];
  error?: string;
}

interface ObsidianArtifactEntry {
  subject_id: string;
  subject_kind?: string;
  book_id: string;
  file_path: string;
  full_content?: string | null;
  append_only_content?: string | null;
  last_content_hash?: string | null;
  last_highlight_created_at?: string | null;
  last_highlight_id?: string | null;
  full_document_text_path?: string | null;
  full_document_text?: string | null;
  last_full_document_hash?: string | null;
}

interface ObsidianSyncNotificationArtifact {
  file_path: string;
  append_content: string;
}

interface ObsidianArtifactManifest {
  version: number;
  run_id: string;
  generated_at: string;
  entries: ObsidianArtifactEntry[];
  sync_notification?: ObsidianSyncNotificationArtifact | null;
}

interface AckSubject {
  subject_id: string;
  status: "success" | "failed";
  error?: string;
  last_content_hash?: string | null;
  last_full_document_hash?: string | null;
}

interface WriteOutcome {
  lastContentHash?: string;
  lastFullDocumentHash?: string;
}

const DEFAULT_SETTINGS: IndelibleSettings = {
  tokenSecretName: "",
  apiBaseUrl: "https://api.useindelible.com",
  baseFolder: "Indelible",
  syncFrequency: "manual",
  syncOnOpen: true,
  resyncDeletedFiles: false,
  reimportConfirmation: true,
  customizeFormattingUrl: "https://app.useindelible.com/preferences/integrations/obsidian",
  pollMaxMinutes: 30,
  failedSubjects: {},
  deletedReimportQueue: [],
  fileToSubjectMap: {}
};

export async function hashText(text: string): Promise<string> {
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

export default class IndeliblePlugin extends Plugin {
  settings: IndelibleSettings = { ...DEFAULT_SETTINGS };
  private statusBar?: HTMLElement;
  private settingTab?: IndelibleSettingTab;
  private syncInFlight = false;
  private syncIntervalId: number | null = null;

  async onload(): Promise<void> {
    await this.loadSettings();
    this.statusBar = this.addStatusBarItem();
    this.updateStatus("Idle");
    this.registerSyncButtonStyles();

    this.settingTab = new IndelibleSettingTab(this.app, this);
    this.addSettingTab(this.settingTab);

    this.addCommand({
      id: "sync-now",
      name: "Sync your data now",
      callback: () => {
        void this.sync({ auto: false });
      }
    });

    this.addCommand({
      id: "customize-formatting",
      name: "Customize formatting",
      callback: () => this.openFormatting()
    });

    this.addCommand({
      id: "delete-and-reimport-current-document",
      name: "Delete and reimport current exported document",
      callback: () => {
        void this.deleteAndReimportCurrentFile();
      }
    });

    this.registerEvent(
      this.app.vault.on("delete", (file) => {
        void this.onVaultDelete(file);
      })
    );

    this.registerEvent(
      this.app.vault.on("rename", (file, oldPath) => {
        void this.onVaultRename(file, oldPath);
      })
    );

    this.registerConfiguredInterval();

    const syncOnOpenDelay = syncOnOpenDelayMs(this.settings.syncOnOpen);
    if (syncOnOpenDelay !== null) {
      window.setTimeout(() => {
        void this.sync({ auto: true });
      }, syncOnOpenDelay);
    }
  }

  async loadSettings(): Promise<void> {
    const loaded = (await this.loadData()) as Partial<IndelibleSettings> | null;
    this.settings = {
      ...DEFAULT_SETTINGS,
      ...loaded,
      failedSubjects: loaded?.failedSubjects ?? {},
      deletedReimportQueue: loaded?.deletedReimportQueue ?? [],
      fileToSubjectMap: loaded?.fileToSubjectMap ?? {}
    };
  }

  async saveSettings(): Promise<void> {
    await this.saveData(this.settings);
  }

  isSyncing(): boolean {
    return this.syncInFlight;
  }

  async sync(options: { auto: boolean }): Promise<void> {
    if (this.syncInFlight) {
      new Notice("Indelible sync is already running.");
      return;
    }
    this.syncInFlight = true;
    this.updateStatus("Syncing");
    this.refreshSettingsTab();

    const ackSubjects: AckSubject[] = [];
    try {
      const token = await this.getToken();
      const status = await this.createRun(token, this.createRunRequest(options.auto));
      const finished = await this.pollRun(token, status.run_id);
      if (finished.error) {
        throw new Error(finished.error);
      }

      for (const artifactId of finished.artifact_ids) {
        const manifest = await this.downloadArtifact(token, artifactId);
        const artifactAck = await this.applyManifest(manifest);
        ackSubjects.push(...artifactAck);
      }

      const acked = await this.ackRun(token, finished.run_id, finished.artifact_ids, ackSubjects);
      if (acked.error && acked.task_status !== "partial_success") {
        throw new Error(acked.error);
      }

      this.clearSuccessfulQueues(ackSubjects);
      this.settings.lastSyncAt = new Date().toISOString();
      const writeCounts = countWriteOutcomes(ackSubjects);
      const failedWrites = ackSubjects.filter((subject) => subject.status === "failed");
      if (writeCounts.failed > 0) {
        const firstError = failedWrites[0]?.error ?? "plugin write failed";
        if (writeCounts.successful === 0) {
          throw new Error(`0 of ${ackSubjects.length} document writes succeeded: ${firstError}`);
        }
        this.settings.lastSyncStatus = `Synced ${writeCounts.successful} documents; ${writeCounts.failed} failed`;
        await this.saveSettings();
        this.updateStatus("Failed");
        new Notice(this.settings.lastSyncStatus);
        return;
      }
      const syncedCount = ackSubjects.length > 0 ? writeCounts.successful : acked.documents_exported;
      this.settings.lastSyncStatus = `Synced ${syncedCount} documents`;
      await this.saveSettings();
      this.updateStatus("Synced");
      new Notice(this.settings.lastSyncStatus);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.settings.lastSyncAt = new Date().toISOString();
      this.settings.lastSyncStatus = `Sync failed: ${message}`;
      await this.saveSettings();
      this.updateStatus("Failed");
      new Notice(this.settings.lastSyncStatus);
    } finally {
      this.syncInFlight = false;
      this.refreshSettingsTab();
    }
  }

  private refreshSettingsTab(): void {
    this.settingTab?.display();
  }

  private registerSyncButtonStyles(): void {
    const styleEl = document.createElement("style");
    styleEl.textContent = `
      @keyframes indelible-sync-spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
      }

      .indelible-sync-button.is-syncing svg {
        animation: indelible-sync-spin 1s linear infinite;
      }

      .indelible-sync-button.is-syncing::before {
        content: "";
        width: 12px;
        height: 12px;
        margin-right: 6px;
        border: 2px solid currentColor;
        border-right-color: transparent;
        border-radius: 999px;
        animation: indelible-sync-spin 1s linear infinite;
      }
    `;
    document.head.appendChild(styleEl);
    this.register(() => styleEl.remove());
  }

  registerConfiguredInterval(): void {
    if (this.syncIntervalId !== null) {
      window.clearInterval(this.syncIntervalId);
      this.syncIntervalId = null;
    }

		const intervalMs = intervalMsForFrequency(this.settings.syncFrequency);
    if (!intervalMs) {
      return;
    }
    const intervalId = window.setInterval(() => {
      void this.sync({ auto: true });
    }, intervalMs);
    this.syncIntervalId = intervalId;
    this.registerInterval(intervalId);
  }

  private async getToken(): Promise<string> {
		const secretStorage = (this.app as App & { secretStorage?: SecretStorageLike }).secretStorage;
		return resolveSecretToken(this.settings.tokenSecretName, secretStorage);
  }

  private apiUrl(path: string): string {
    return `${this.settings.apiBaseUrl.replace(/\/+$/, "")}${path}`;
  }

  private async requestJson<T>(
    token: string,
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const response = await requestUrl({
      url: this.apiUrl(path),
      method,
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json"
      },
      body: body === undefined ? undefined : JSON.stringify(body),
      throw: false
    });
    if (response.status < 200 || response.status >= 300) {
      throw new Error(`${method} ${path} failed with HTTP ${response.status}: ${response.text}`);
    }
    return response.json as T;
  }

  private async createRun(
    token: string,
    body: { parent_folder_deleted: boolean; auto: boolean; force_subject_ids: string[] }
  ): Promise<ObsidianRunStatusResponse> {
    return this.requestJson<ObsidianRunStatusResponse>(
      token,
      "POST",
      "/api/v1/export/obsidian/runs",
      body
    );
  }

  private async pollRun(token: string, runId: string): Promise<ObsidianRunStatusResponse> {
    const start = Date.now();
    const cap = Math.max(1, this.settings.pollMaxMinutes ?? 30);
    const maxDurationMs = cap * 60_000;
    let delay = 1000;
    while (Date.now() - start < maxDurationMs) {
      const status = await this.requestJson<ObsidianRunStatusResponse>(
        token,
        "GET",
        `/api/v1/export/obsidian/runs/${runId}`
      );
      if (status.is_finished) {
        return status;
      }
      await sleep(delay);
      delay = Math.min(Math.floor(delay * 1.5), 15_000);
    }
    throw new Error(
      `Obsidian export run ${runId} did not finish within ${cap} minutes. ` +
        "Increase the poll timeout in plugin settings or check the Indelible worker."
    );
  }

  private async downloadArtifact(token: string, artifactId: string): Promise<ObsidianArtifactManifest> {
    const response = await requestUrl({
      url: this.apiUrl(`/api/v1/export/obsidian/artifacts/${artifactId}`),
      method: "GET",
      headers: {
        Authorization: `Bearer ${token}`
      },
      throw: false
    });
    if (response.status < 200 || response.status >= 300) {
      throw new Error(`Artifact ${artifactId} failed with HTTP ${response.status}: ${response.text}`);
    }
		return parseArtifactManifestZip(artifactId, response.arrayBuffer) as ObsidianArtifactManifest;
  }

  private async ackRun(
    token: string,
    runId: string,
    artifactIds: string[],
    subjects: AckSubject[]
  ): Promise<ObsidianRunStatusResponse> {
    return this.requestJson<ObsidianRunStatusResponse>(
      token,
      "POST",
      `/api/v1/export/obsidian/runs/${runId}/ack`,
      {
        artifact_ids: artifactIds,
        subjects
      }
    );
  }

  private async refreshSubjects(token: string, subjectIds: string[], reason: string): Promise<void> {
    if (subjectIds.length === 0) {
      return;
    }
    await this.requestJson<{ queued: number }>(token, "POST", "/api/v1/export/obsidian/refresh", {
      subject_ids: subjectIds,
      reason
    });
  }

  private async applyManifest(manifest: ObsidianArtifactManifest): Promise<AckSubject[]> {
    const ackSubjects: AckSubject[] = [];
    const duplicatePaths = findDuplicateVaultPaths(manifest.entries, this.settings.baseFolder);
    if (duplicatePaths.length > 0) {
      const message = `Artifact contains duplicate vault paths: ${duplicatePaths
        .slice(0, 5)
        .join(", ")}${duplicatePaths.length > 5 ? ", ..." : ""}`;
      for (const entry of manifest.entries) {
        this.settings.failedSubjects[entry.subject_id] = {
          subjectId: entry.subject_id,
          error: message,
          lastAttemptAt: new Date().toISOString()
        };
        ackSubjects.push({ subject_id: entry.subject_id, status: "failed", error: message });
      }
      await this.saveSettings();
      return ackSubjects;
    }

    for (const entry of manifest.entries) {
      try {
        const outcome = await this.applyEntry(entry);
        delete this.settings.failedSubjects[entry.subject_id];
        ackSubjects.push({
          subject_id: entry.subject_id,
          status: "success",
          last_content_hash: outcome.lastContentHash ?? null,
          last_full_document_hash: outcome.lastFullDocumentHash ?? null
        });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        this.settings.failedSubjects[entry.subject_id] = {
          subjectId: entry.subject_id,
          error: message,
          lastAttemptAt: new Date().toISOString()
        };
        ackSubjects.push({ subject_id: entry.subject_id, status: "failed", error: message });
      }
    }

    const reconciliation = reconcileManifestWrites({
      entries: manifest.entries,
      ackSubjects: ackSubjects,
      baseFolder: this.settings.baseFolder,
      fileToSubjectMap: this.settings.fileToSubjectMap,
      fileExists: (path) => this.getFile(path) !== null
    });
    if (!reconciliation.ok) {
      const message = reconciliation.error ?? "Obsidian write reconciliation failed.";
      for (const ackSubject of ackSubjects) {
        if (ackSubject.status !== "success") {
          continue;
        }
        ackSubject.status = "failed";
        ackSubject.error = message;
        this.settings.failedSubjects[ackSubject.subject_id] = {
          subjectId: ackSubject.subject_id,
          error: message,
          lastAttemptAt: new Date().toISOString()
        };
      }
    }

    if (
      manifest.sync_notification?.append_content &&
      shouldAppendSyncNotification(manifest.entries.length, ackSubjects)
    ) {
      const path = rewriteBaseFolder(manifest.sync_notification.file_path, this.settings.baseFolder);
      await this.appendToPath(path, manifest.sync_notification.append_content);
    }
    await this.saveSettings();
    return ackSubjects;
  }

  private async applyEntry(entry: ObsidianArtifactEntry): Promise<WriteOutcome> {
    const localPath = rewriteBaseFolder(entry.file_path, this.settings.baseFolder);
    const existing = this.getFile(localPath);
    const mapped = this.settings.fileToSubjectMap[localPath];
    const fullContent = entry.full_content?.trim();
    const appendOnlyContent = entry.append_only_content?.trim();
    let lastContentHash: string | undefined;
    let existingContent: string | undefined;
    let existingHash: string | undefined;

    if (existing) {
      existingContent = await this.app.vault.read(existing);
      existingHash = await hashText(existingContent);
    }

    const notePlan = planNoteContentWrite({
      path: localPath,
      existingContent,
      existingHash,
      mappedLastContentHash: mapped?.lastContentHash,
      fullContent,
      appendOnlyContent
    });

    if (notePlan.action === "error") {
      throw new Error(notePlan.error);
    } else if (notePlan.action === "create") {
      await this.writeNewFile(localPath, notePlan.content);
      if (notePlan.ackContentHash) {
        lastContentHash = await hashText(notePlan.content);
      }
    } else if (notePlan.action === "replace" || notePlan.action === "append") {
      if (!existing) {
        throw new Error(`${localPath} does not exist for ${notePlan.action}.`);
      }
      await this.app.vault.modify(existing, notePlan.content);
      if (notePlan.ackContentHash) {
        lastContentHash = await hashText(notePlan.content);
      }
    }

    const lastFullDocumentHash = await this.writeFullDocumentCompanion(entry);

    this.settings.fileToSubjectMap[localPath] = {
      subjectId: entry.subject_id,
      subjectKind: entry.subject_kind ?? mapped?.subjectKind,
      bookId: entry.book_id,
      serverPath: entry.file_path,
      localPath,
      fullDocumentPath: entry.full_document_text_path
        ? rewriteBaseFolder(entry.full_document_text_path, this.settings.baseFolder)
        : mapped?.fullDocumentPath,
      lastContentHash: lastContentHash ?? mapped?.lastContentHash,
      lastFullDocumentHash: lastFullDocumentHash ?? mapped?.lastFullDocumentHash,
      updatedAt: new Date().toISOString()
    };
    return { lastContentHash, lastFullDocumentHash };
  }

	private async writeFullDocumentCompanion(
		entry: ObsidianArtifactEntry
	): Promise<string | undefined> {
		const localPath = rewriteBaseFolder(entry.file_path, this.settings.baseFolder);
		const fullPath = entry.full_document_text_path
			? rewriteBaseFolder(entry.full_document_text_path, this.settings.baseFolder)
			: undefined;
		const existing = fullPath ? this.getFile(fullPath) : null;
		const mapped = this.settings.fileToSubjectMap[localPath];
		let existingHash: string | undefined;

		if (existing) {
			const existingContent = await this.app.vault.read(existing);
			existingHash = await hashText(existingContent);
		}

		const companionPlan = planFullDocumentCompanionWrite({
			fullDocumentTextPath: fullPath,
			fullDocumentText: entry.full_document_text,
			existingHash,
			mappedLastFullDocumentHash: mapped?.lastFullDocumentHash
		});

		if (companionPlan.action === "create" && fullPath) {
			await this.writeNewFile(fullPath, companionPlan.content);
			return hashText(companionPlan.content);
		}
		if (companionPlan.action === "replace" && existing) {
			await this.app.vault.modify(existing, companionPlan.content);
			return hashText(companionPlan.content);
		}
		return undefined;
	}

  private async appendToPath(path: string, content: string): Promise<string | undefined> {
    const existing = this.getFile(path);
    if (!existing) {
      await this.writeNewFile(path, content);
      return hashText(content);
    }
    const previous = await this.app.vault.read(existing);
    const previousHash = await hashText(previous);
    const nextContent = joinMarkdown([previous, content]);
    await this.app.vault.modify(existing, nextContent);
    const mapped = this.settings.fileToSubjectMap[path];
    if (mapped?.lastContentHash && previousHash === mapped.lastContentHash) {
      return hashText(nextContent);
    }
    return undefined;
  }

  private async writeNewFile(path: string, content: string): Promise<void> {
    await this.ensureParentFolders(path);
    await this.app.vault.create(path, content);
  }

  private getFile(path: string): TFile | null {
    const abstractFile = this.app.vault.getAbstractFileByPath(path);
    if (!abstractFile) {
      return null;
    }
    if (abstractFile instanceof TFile) {
      return abstractFile;
    }
    throw new Error(`${path} exists but is not a file.`);
  }

  private async ensureParentFolders(path: string): Promise<void> {
    const parts = normalizePath(path).split("/");
    parts.pop();
    let current = "";
    for (const part of parts) {
      current = current ? `${current}/${part}` : part;
      const existing = this.app.vault.getAbstractFileByPath(current);
      if (existing instanceof TFolder) {
        continue;
      }
      if (existing) {
        throw new Error(`${current} exists but is not a folder.`);
      }
      await this.app.vault.createFolder(current);
    }
  }

	private createRunRequest(auto: boolean): { parent_folder_deleted: boolean; auto: boolean; force_subject_ids: string[] } {
		return createRunRequest({
			auto,
			baseFolder: normalizePath(this.settings.baseFolder || "Indelible"),
			fileToSubjectMap: this.settings.fileToSubjectMap,
			failedSubjects: this.settings.failedSubjects,
			deletedReimportQueue: this.settings.deletedReimportQueue,
			resyncDeletedFiles: this.settings.resyncDeletedFiles,
			pathExists: (path) => this.app.vault.getAbstractFileByPath(path) !== null
		});
	}

  private clearSuccessfulQueues(ackSubjects: AckSubject[]): void {
    const successful = new Set(
      ackSubjects.filter((subject) => subject.status === "success").map((subject) => subject.subject_id)
    );
    this.settings.deletedReimportQueue = this.settings.deletedReimportQueue.filter(
      (subjectId) => !successful.has(subjectId)
    );
    for (const subjectId of successful) {
      delete this.settings.failedSubjects[subjectId];
    }
  }

  private async onVaultDelete(file: TAbstractFile): Promise<void> {
    const mapEntry = this.settings.fileToSubjectMap[file.path];
    if (!mapEntry) {
      return;
    }
    delete this.settings.fileToSubjectMap[file.path];
    this.settings.deletedReimportQueue = nextDeletedReimportQueue(
      this.settings.deletedReimportQueue,
      mapEntry.subjectId,
      this.settings.resyncDeletedFiles
    );
    await this.saveSettings();
  }

  private async onVaultRename(file: TAbstractFile, oldPath: string): Promise<void> {
    const mapEntry = this.settings.fileToSubjectMap[oldPath];
    if (!mapEntry) {
      return;
    }
		this.settings.fileToSubjectMap = moveSubjectFileMapEntry(
			this.settings.fileToSubjectMap,
			oldPath,
			file.path,
			new Date().toISOString()
		);
		await this.saveSettings();

    try {
      const token = await this.getToken();
      await this.requestJson<{ subject_id: string; new_path: string }>(
        token,
        "POST",
        "/api/v1/export/obsidian/rename",
        { subject_id: mapEntry.subjectId, new_path: file.path }
      );
    } catch (err) {
      // Non-fatal: the local map keeps the plugin usable, but server cursor
      // state may remain stale until a later successful rename or export.
      console.warn("Indelible: failed to record rename on the server", err);
    }
  }

  private async deleteAndReimportCurrentFile(): Promise<void> {
    const file = this.app.workspace.getActiveFile();
    if (!file) {
      new Notice("Open an exported Indelible file first.");
      return;
    }
    const mapEntry = this.settings.fileToSubjectMap[file.path];
    if (!mapEntry) {
      new Notice("This file is not tracked as an Indelible export.");
      return;
    }
    if (
      this.settings.reimportConfirmation &&
      !window.confirm("Delete this exported note and queue a full Indelible reimport?")
    ) {
      return;
    }
		this.settings.deletedReimportQueue = explicitReimportQueue(
			this.settings.deletedReimportQueue,
			mapEntry.subjectId
		);
    delete this.settings.fileToSubjectMap[file.path];
    await this.app.vault.delete(file);
    await this.saveSettings();

    try {
      const token = await this.getToken();
      await this.refreshSubjects(token, [mapEntry.subjectId], "current_document_reimport");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.settings.failedSubjects[mapEntry.subjectId] = {
        subjectId: mapEntry.subjectId,
        error: message,
        lastAttemptAt: new Date().toISOString()
      };
      await this.saveSettings();
    }
    new Notice("Queued Indelible reimport for this document.");
  }

  openFormatting(): void {
    window.open(this.settings.customizeFormattingUrl, "_blank");
  }

  updateStatus(status: string): void {
    this.statusBar?.setText(`Indelible: ${status}`);
  }
}

class IndelibleSettingTab extends PluginSettingTab {
  plugin: IndeliblePlugin;

  constructor(app: App, plugin: IndeliblePlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  display(): void {
    const { containerEl } = this;
    containerEl.empty();

    containerEl.createEl("h2", { text: "Indelible" });

    new Setting(containerEl)
      .setName("Token")
      .setDesc("Select a token stored in Obsidian SecretStorage.")
      .addComponent((el) =>
        new SecretComponent(this.app, el)
          .setValue(this.plugin.settings.tokenSecretName)
          .onChange((value) => {
            this.plugin.settings.tokenSecretName = value;
            void this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName("API base URL")
      .addText((text) =>
        text
          .setPlaceholder("https://api.indelible.app")
          .setValue(this.plugin.settings.apiBaseUrl)
          .onChange((value) => {
            this.plugin.settings.apiBaseUrl = value.trim() || DEFAULT_SETTINGS.apiBaseUrl;
            void this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName("Base folder")
      .addText((text) =>
        text
          .setPlaceholder("Indelible")
          .setValue(this.plugin.settings.baseFolder)
          .onChange((value) => {
            this.plugin.settings.baseFolder = value.trim() || DEFAULT_SETTINGS.baseFolder;
            void this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName("Sync interval")
      .setDesc("Manual still syncs on open when Sync on open is enabled.")
      .addDropdown((dropdown) =>
        dropdown
          .addOptions({
            manual: "Manual",
            "1h": "Every hour",
            "12h": "Every 12 hours",
            "24h": "Every 24 hours",
            week: "Weekly"
          })
          .setValue(this.plugin.settings.syncFrequency)
          .onChange((value) => {
            this.plugin.settings.syncFrequency = value as SyncFrequency;
            this.plugin.registerConfiguredInterval();
            void this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName("Sync on open")
      .addToggle((toggle) =>
        toggle.setValue(this.plugin.settings.syncOnOpen).onChange((value) => {
          this.plugin.settings.syncOnOpen = value;
          void this.plugin.saveSettings();
        })
      );

    new Setting(containerEl)
      .setName("Resync deleted files")
      .addToggle((toggle) =>
        toggle.setValue(this.plugin.settings.resyncDeletedFiles).onChange((value) => {
          this.plugin.settings.resyncDeletedFiles = value;
          void this.plugin.saveSettings();
        })
      );

    new Setting(containerEl)
      .setName("Confirm current-file reimports")
      .addToggle((toggle) =>
        toggle.setValue(this.plugin.settings.reimportConfirmation).onChange((value) => {
          this.plugin.settings.reimportConfirmation = value;
          void this.plugin.saveSettings();
        })
      );

    new Setting(containerEl)
      .setName("Sync timeout")
      .setDesc(
        "Maximum minutes to wait for a sync run to complete before giving up. Larger libraries may need more."
      )
      .addText((text) =>
        text
          .setPlaceholder(String(DEFAULT_SETTINGS.pollMaxMinutes))
          .setValue(String(this.plugin.settings.pollMaxMinutes))
          .onChange((value) => {
            const parsed = Number.parseInt(value, 10);
            this.plugin.settings.pollMaxMinutes = Number.isFinite(parsed) && parsed > 0
              ? parsed
              : DEFAULT_SETTINGS.pollMaxMinutes;
            void this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName("Customize formatting")
      .addText((text) =>
        text
          .setPlaceholder(DEFAULT_SETTINGS.customizeFormattingUrl)
          .setValue(this.plugin.settings.customizeFormattingUrl)
          .onChange((value) => {
            this.plugin.settings.customizeFormattingUrl =
              value.trim() || DEFAULT_SETTINGS.customizeFormattingUrl;
            void this.plugin.saveSettings();
          })
      )
      .addButton((button) =>
        button.setButtonText("Open").onClick(() => this.plugin.openFormatting())
      );

    new Setting(containerEl)
      .setName("Sync")
      .setDesc(this.plugin.settings.lastSyncStatus ?? "No sync has run yet.")
      .addButton((button) => {
        const syncing = this.plugin.isSyncing();
        button
          .setButtonText(syncing ? "Syncing..." : "Sync now")
          .setDisabled(syncing)
          .setTooltip(syncing ? "A sync is already running." : "Start a sync now.")
          .onClick(() => {
            void this.plugin.sync({ auto: false });
          });

        button.buttonEl.classList.add("indelible-sync-button");
        if (syncing) {
          button.setIcon("loader");
          button.buttonEl.classList.add("is-syncing");
          button.buttonEl.setAttribute("aria-busy", "true");
        } else {
          button.setCta();
          button.buttonEl.removeAttribute("aria-busy");
        }
      });

    const failedCount = Object.keys(this.plugin.settings.failedSubjects).length;
    new Setting(containerEl)
      .setName("Failed subjects")
      .setDesc(`${failedCount} failed subject${failedCount === 1 ? "" : "s"} queued.`)
      .addButton((button) =>
        button
          .setButtonText("Retry now")
          .setDisabled(failedCount === 0 || this.plugin.isSyncing())
          .setTooltip(
            failedCount === 0
              ? "No failed subjects to retry."
              : this.plugin.isSyncing()
                ? "A sync is already running."
                : "Retry queued failed subjects."
          )
          .onClick(() => {
            if (failedCount === 0) {
              return;
            }
            void this.plugin.sync({ auto: false });
          })
      );
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}
