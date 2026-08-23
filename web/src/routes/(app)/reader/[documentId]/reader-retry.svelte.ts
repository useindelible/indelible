import * as apiSdk from '$lib/api';
import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';
import type { MessageKey, Translate, TranslateOptions } from '$lib/i18n';

import { shouldReprocessReaderPreparation } from './reader-page-model';

export class ReaderRetryController {
	error = $state<string | null>(null);
	status = $state<string | null>(null);
	outcome = $state<string | null>(null);
	state = $state<'idle' | 'submitting' | 'queued' | 'cooldown'>('idle');
	pollVisible = $state(false);
	#cooldownTimer: ReturnType<typeof setTimeout> | undefined;
	#awaitingQueuedResult = false;
	#requestEpoch = 0;

	constructor(private readonly translate: Translate) {}

	get label(): string {
		if (this.state === 'submitting') return this.translate('reader_retry_queuing');
		if (this.state === 'queued') return this.translate('reader_retry_status_queued');
		if (this.state === 'cooldown') return this.translate('reader_retry_cooling_down');
		return this.translate('reader_retry');
	}

	get disabled(): boolean {
		return this.state !== 'idle';
	}

	onPreparationReady(ready: boolean) {
		if (!ready) return;
		if (this.#awaitingQueuedResult) {
			this.#awaitingQueuedResult = false;
			this.outcome = this.translate('reader_retry_outcome_ready');
		}
		if (this.state !== 'idle') {
			this.clearCooldown();
			this.state = 'idle';
			this.status = null;
		}
	}

	async submit(options: {
		documentId: string;
		item: DocumentListEntry | null;
		assets: DocumentReaderAssetResponse[];
		onRetryPolling: () => void;
	}) {
		if (this.disabled) return;
		const requestEpoch = ++this.#requestEpoch;
		this.error = null;
		this.status = null;
		this.outcome = null;
		this.#awaitingQueuedResult = false;
		if (shouldReprocessReaderPreparation(options.item, options.assets)) {
			this.state = 'submitting';
			try {
				const { data } = await apiSdk.reprocessDocument({
					path: { document_id: options.documentId }
				});
				if (requestEpoch !== this.#requestEpoch) return;
				if (!data) throw new Error('Reprocess response was empty');
				if (data.queued) {
					this.#awaitingQueuedResult = true;
					this.hold('queued', 'reader_retry_queued', 5 * 60);
				} else if (data.retry_after_seconds) {
					this.hold('cooldown', 'reader_retry_available', data.retry_after_seconds, {
						values: { seconds: data.retry_after_seconds }
					});
				} else {
					this.hold('queued', 'reader_retry_already_running', 30);
				}
			} catch {
				if (requestEpoch !== this.#requestEpoch) return;
				this.state = 'idle';
				this.error = this.translate('reader_retry_error');
				return;
			}
		}
		options.onRetryPolling();
	}

	reset() {
		this.#requestEpoch += 1;
		this.clearCooldown();
		this.#awaitingQueuedResult = false;
		this.state = 'idle';
		this.error = null;
		this.status = null;
		this.outcome = null;
	}

	destroy() {
		this.reset();
	}

	private hold(
		state: 'queued' | 'cooldown',
		statusKey: MessageKey,
		retryAfterSeconds: number,
		options?: TranslateOptions
	) {
		this.clearCooldown();
		this.state = state;
		this.status = this.translate(statusKey, options);
		this.#cooldownTimer = setTimeout(() => {
			this.state = 'idle';
			this.status = null;
			this.#cooldownTimer = undefined;
		}, retryAfterSeconds * 1000);
	}

	private clearCooldown() {
		if (this.#cooldownTimer) clearTimeout(this.#cooldownTimer);
		this.#cooldownTimer = undefined;
	}
}
