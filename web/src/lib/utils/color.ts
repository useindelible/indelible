const VALID_HEX = /^#[0-9a-fA-F]{3,8}$/;
const PALETTE = ['#FFD600', '#0A84FF', '#34C759', '#FF2D55', '#AF52DE'];

export function sanitizeColor(color: string | null | undefined): string | undefined {
	if (!color) return undefined;
	if (VALID_HEX.test(color)) return color;
	if (PALETTE.includes(color.toUpperCase())) return color;
	return undefined;
}
