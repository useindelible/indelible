export function relativeTime(iso: string | null | undefined): string | null {
	if (!iso) return null;
	const timestamp = new Date(iso).getTime();
	if (Number.isNaN(timestamp)) return null;

	const diffMs = Date.now() - timestamp;
	if (diffMs < 60_000) return 'just now';

	const minutes = Math.floor(diffMs / 60_000);
	if (minutes < 60) return minutes === 1 ? '1 minute ago' : `${minutes} minutes ago`;

	const hours = Math.floor(minutes / 60);
	if (hours < 24) return hours === 1 ? '1 hour ago' : `${hours} hours ago`;

	const days = Math.floor(hours / 24);
	if (days < 30) return days === 1 ? '1 day ago' : `${days} days ago`;

	const months = Math.floor(days / 30);
	if (months < 12) return months === 1 ? '1 month ago' : `${months} months ago`;

	const years = Math.floor(months / 12);
	return years === 1 ? '1 year ago' : `${years} years ago`;
}
