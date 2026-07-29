import { describe, expect, test, vi } from 'vitest';

import { createProgressSaver, type ProgressSaveInput } from '$lib/components/reader/progress-saver';

describe('progress saver', () => {
	test('saves the latest progress after the idle window', async () => {
		vi.useFakeTimers();
		const saves: ProgressSaveInput[] = [];
		const saver = createProgressSaver(async (body) => {
			saves.push(body);
		});

		saver.update({ progress_percent: 42.123 });
		await vi.advanceTimersByTimeAsync(799);
		expect(saves).toEqual([]);

		await vi.advanceTimersByTimeAsync(1);
		expect(saves).toEqual([
			{ progress_percent: 42.12, chapter_locator: null, chapter_offset: null }
		]);

		vi.useRealTimers();
	});

	test('allows resume progress to move backward', async () => {
		const saves: ProgressSaveInput[] = [];
		const saver = createProgressSaver(async (body) => {
			saves.push(body);
		});

		saver.update({ progress_percent: 80 });
		await saver.flush();
		saver.update({ progress_percent: 25 });
		await saver.flush();

		expect(saves.map((save) => save.progress_percent)).toEqual([80, 25]);
	});

	test('throttles saves while progress keeps changing', async () => {
		vi.useFakeTimers();
		const saves: ProgressSaveInput[] = [];
		const saver = createProgressSaver(async (body) => {
			saves.push(body);
		});

		saver.update({ progress_percent: 10 });
		for (let step = 1; step <= 7; step += 1) {
			await vi.advanceTimersByTimeAsync(700);
			saver.update({ progress_percent: 10 + step });
		}
		expect(saves).toEqual([]);

		await vi.advanceTimersByTimeAsync(200);
		saver.update({ progress_percent: 18 });
		expect(saves.map((save) => save.progress_percent)).toEqual([18]);

		vi.useRealTimers();
	});

	test('keeps a backward update while a higher save is in flight', async () => {
		let resolveHigherSave: (() => void) | undefined;
		const saves: ProgressSaveInput[] = [];
		const saver = createProgressSaver(async (body) => {
			saves.push(body);
			if (body.progress_percent === 60) {
				await new Promise<void>((resolve) => {
					resolveHigherSave = resolve;
				});
			}
		});

		saver.update({ progress_percent: 50 });
		await saver.flush();

		saver.update({ progress_percent: 60 });
		const higherFlush = saver.flush();
		saver.update({ progress_percent: 50 });

		resolveHigherSave?.();
		await higherFlush;

		expect(saves.map((save) => save.progress_percent)).toEqual([50, 60, 50]);
	});
});
