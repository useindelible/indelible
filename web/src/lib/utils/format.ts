import { t } from '$lib/i18n';
import { get } from 'svelte/store';

export function formatReadingTime(minutes: number): string {
	if (minutes < 60) {
		return get(t)('common_reading_time_minutes', { values: { minutes } });
	}

	const hours = Math.floor(minutes / 60);
	const remainingMinutes = minutes % 60;
	return remainingMinutes > 0
		? get(t)('common_reading_time_hours_minutes', {
				values: { hours, minutes: remainingMinutes }
			})
		: get(t)('common_reading_time_hours', { values: { hours } });
}
