const SPOKEN_SELECTOR = 'h1, h2, h3, h4, h5, h6, p, blockquote, li, figcaption, caption';

type TtsHighlightState = {
	highlightedEl: HTMLElement | null;
	speakingPillEl: HTMLElement | null;
};

export function collectTtsSpokenElements(articleBodyEl: HTMLDivElement): HTMLElement[] {
	return Array.from(articleBodyEl.querySelectorAll<HTMLElement>(SPOKEN_SELECTOR)).filter(
		(element) =>
			!hasSpokenAncestor(element) &&
			!isExcludedElement(element) &&
			!hasExcludedAncestor(element) &&
			normalizedText(element).length > 0
	);
}

export function clearTtsHighlight(
	currentHighlightedEl: HTMLElement | null,
	currentSpeakingPillEl: HTMLElement | null
): void {
	currentSpeakingPillEl?.remove();
	currentHighlightedEl?.classList.remove('tts-active');
}

export function setTtsHighlight(
	domElements: HTMLElement[],
	elementIndex: number,
	instant = false
): TtsHighlightState {
	const element = domElements[elementIndex];
	if (!element) return { highlightedEl: null, speakingPillEl: null };
	if (element.offsetParent === null) {
		return { highlightedEl: null, speakingPillEl: null };
	}
	element.classList.add('tts-active');
	const speakingPillEl = attachSpeakingPill(element);
	scrollHighlightIntoView(element, instant);
	return { highlightedEl: element, speakingPillEl };
}

function hasSpokenAncestor(element: HTMLElement): boolean {
	return element.parentElement?.closest(SPOKEN_SELECTOR) != null;
}

function hasExcludedAncestor(element: HTMLElement): boolean {
	let current = element.parentElement;
	while (current) {
		if (isExcludedElement(current)) {
			return true;
		}
		current = current.parentElement;
	}
	return false;
}

function isExcludedElement(element: HTMLElement): boolean {
	const tag = element.tagName.toLowerCase();
	return (
		[
			'script',
			'style',
			'nav',
			'header',
			'footer',
			'aside',
			'form',
			'button',
			'input',
			'select',
			'textarea',
			'noscript'
		].includes(tag) ||
		element.hasAttribute('hidden') ||
		element.getAttribute('aria-hidden') === 'true'
	);
}

function normalizedText(element: HTMLElement): string {
	return (element.textContent ?? '').split(/\s+/).filter(Boolean).join(' ');
}

function scrollHighlightIntoView(element: HTMLElement, instant = false) {
	const scrollContainer = findScrollParent(element);
	if (!scrollContainer) return;
	const containerRect = scrollContainer.getBoundingClientRect();
	const elementRect = element.getBoundingClientRect();
	const currentScroll = scrollContainer.scrollTop;
	const offsetWithinContainer = elementRect.top - containerRect.top + currentScroll;
	const target = offsetWithinContainer - (scrollContainer.clientHeight - element.offsetHeight) / 2;
	const clamped = Math.max(
		0,
		Math.min(scrollContainer.scrollHeight - scrollContainer.clientHeight, target)
	);
	scrollContainer.scrollTo({ top: clamped, behavior: instant ? 'instant' : 'smooth' });
}

function findScrollParent(element: HTMLElement): HTMLElement | null {
	let current: HTMLElement | null = element.parentElement;
	while (current) {
		const style = window.getComputedStyle(current);
		if (/(auto|scroll)/.test(style.overflowY)) {
			return current;
		}
		current = current.parentElement;
	}
	return null;
}

function attachSpeakingPill(element: HTMLElement): HTMLElement {
	const pill = document.createElement('span');
	pill.className = 'tts-speaking-pill';
	pill.setAttribute('aria-live', 'polite');
	const waves = document.createElement('span');
	waves.className = 'tts-pill-waves';
	waves.setAttribute('aria-hidden', 'true');
	waves.append(document.createElement('span'));
	waves.append(document.createElement('span'));
	waves.append(document.createElement('span'));
	pill.append(waves);
	pill.append(document.createTextNode('Speaking'));
	element.prepend(pill);
	return pill;
}
