import * as apiSdk from '$lib/api';
import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';

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

	get label(): string {
		if (this.state === 'submitting') return 'Queuing...';
		if (this.state === 'queued') return 'Queued';
		if (this.state === 'cooldown') return 'Cooling down';
		return 'Retry';
	}

	get disabled(): boolean {
		return this.state !== 'idle';
	}

	onPreparationReady(ready: boolean) {
		if (!ready) return;
		if (this.#awaitingQueuedResult) {
			this.#awaitingQueuedResult = false;
			this.outcome = 'Readable content is ready.';
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
					this.hold('queued', 'Reprocessing queued.', 5 * 60);
				} else if (data.retry_after_seconds) {
					this.hold(
						'cooldown',
						`Retry available in ${data.retry_after_seconds} seconds.`,
						data.retry_after_seconds
					);
				} else {
					this.hold('queued', 'Reprocessing is already running.', 30);
				}
			} catch {
				if (requestEpoch !== this.#requestEpoch) return;
				this.state = 'idle';
				this.error = 'Could not queue reprocessing. Try again.';
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

	private hold(state: 'queued' | 'cooldown', status: string, retryAfterSeconds: number) {
		this.clearCooldown();
		this.state = state;
		this.status = status;
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
