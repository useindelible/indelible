import * as apiSdk from '$lib/api';
import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';

import { shouldReprocessReaderPreparation } from './reader-page-model';

export class ReaderRetryController {
	error = $state<string | null>(null);
	status = $state<string | null>(null);
	state = $state<'idle' | 'submitting' | 'queued' | 'cooldown'>('idle');
	pollVisible = $state(false);
	#cooldownTimer: ReturnType<typeof setTimeout> | undefined;

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
		if (!ready || this.state === 'idle') return;
		this.clearCooldown();
		this.state = 'idle';
		this.status = null;
	}

	async submit(options: {
		documentId: string;
		item: DocumentListEntry | null;
		assets: DocumentReaderAssetResponse[];
		onRetryPolling: () => void;
	}) {
		if (this.disabled) return;
		this.error = null;
		this.status = null;
		if (shouldReprocessReaderPreparation(options.item, options.assets)) {
			this.state = 'submitting';
			try {
				const { data } = await apiSdk.reprocessDocument({
					path: { document_id: options.documentId }
				});
				if (!data) throw new Error('Reprocess response was empty');
				if (data.queued) {
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
				this.state = 'idle';
				this.error = 'Could not queue reprocessing. Try again.';
				return;
			}
		}
		options.onRetryPolling();
	}

	destroy() {
		this.clearCooldown();
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
