const DEFAULT_SCROLL_EPSILON_PX = 1;

type ScrollMetrics = {
	scrollTop: number;
	scrollHeight: number;
	clientHeight: number;
};

export function hasScrollableOverflow(
	{ scrollHeight, clientHeight }: Pick<ScrollMetrics, 'scrollHeight' | 'clientHeight'>,
	thresholdPx = DEFAULT_SCROLL_EPSILON_PX
): boolean {
	return scrollHeight - clientHeight > thresholdPx;
}

export function scrollProgressPercent(
	metrics: ScrollMetrics,
	thresholdPx = DEFAULT_SCROLL_EPSILON_PX
): number {
	if (!hasScrollableOverflow(metrics, thresholdPx)) return 100;

	const maxScroll = metrics.scrollHeight - metrics.clientHeight;
	return Math.min(100, Math.max(0, (metrics.scrollTop / maxScroll) * 100));
}
