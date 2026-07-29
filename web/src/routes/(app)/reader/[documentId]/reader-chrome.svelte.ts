import { afterNavigate } from '$app/navigation';

export class ReaderChromeController {
	backHref = $state<string | null>(null);
	showDetailPanel = $state(true);
	compactDetailOpen = $state(false);

	constructor() {
		afterNavigate((navigation) => {
			if (navigation.from?.url) {
				this.backHref = navigation.from.url.pathname + navigation.from.url.search;
			}
		});
	}

	detailOpen(isCompact: boolean): boolean {
		return isCompact ? this.compactDetailOpen : this.showDetailPanel;
	}

	toggleDetailPanel(isCompact: boolean) {
		if (isCompact) {
			this.compactDetailOpen = !this.compactDetailOpen;
		} else {
			this.showDetailPanel = !this.showDetailPanel;
		}
	}
}
