import { locale, t } from '$lib/i18n';
import { get } from 'svelte/store';

export function relativeTime(
	iso: string | null | undefined,
	now: number = Date.now()
): string | null {
	if (!iso) return null;
	const timestamp = new Date(iso).getTime();
	if (Number.isNaN(timestamp)) return null;

	const diffMs = now - timestamp;
	if (diffMs < 60_000) return get(t)('common_just_now');

	const minutes = Math.floor(diffMs / 60_000);
	const hours = Math.floor(diffMs / 3_600_000);
	const days = Math.floor(diffMs / 86_400_000);
	const months = Math.floor(days / 30);
	const [value, unit] =
		minutes < 60
			? [minutes, 'minute']
			: hours < 24
				? [hours, 'hour']
				: days < 30
					? [days, 'day']
					: months < 12
						? [months, 'month']
						: [Math.floor(months / 12), 'year'];

	return new Intl.RelativeTimeFormat(get(locale) ?? 'en', { numeric: 'always' }).format(
		-value,
		unit as Intl.RelativeTimeFormatUnit
	);
}
