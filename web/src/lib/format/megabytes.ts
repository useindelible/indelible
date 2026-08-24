/**
 * Render a byte count as megabytes: whole numbers stay integers, everything
 * else keeps a single decimal so a 1.5 MiB cap reads "1.5" rather than "2".
 */
export function formatMegabytes(bytes: number): string {
	const megabytes = bytes / (1024 * 1024);
	return Number.isInteger(megabytes) ? megabytes.toString() : megabytes.toFixed(1);
}
