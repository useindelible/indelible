export function isApplePlatform(): boolean {
	return /Mac|iPhone|iPad|iPod/.test(navigator.platform);
}
