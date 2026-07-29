import { browser } from '$app/environment';

export type Breakpoint = 'xs' | 'phablet' | 'tablet' | 'desktop';

let breakpoint = $state<Breakpoint>('desktop');
let mobileNavOpen = $state(false);

// Singleton viewport tracker: listeners live for the app lifetime, matching the
// breakpoints LibraryShell has always used (600 / 900 / 1100).
if (browser) {
	const mqXs = window.matchMedia('(max-width: 599px)');
	const mqPhablet = window.matchMedia('(min-width: 600px) and (max-width: 899px)');
	const mqTablet = window.matchMedia('(min-width: 900px) and (max-width: 1099px)');

	const update = () => {
		if (mqXs.matches) {
			breakpoint = 'xs';
		} else if (mqPhablet.matches) {
			breakpoint = 'phablet';
		} else if (mqTablet.matches) {
			breakpoint = 'tablet';
		} else {
			breakpoint = 'desktop';
		}
		if (breakpoint !== 'xs') {
			mobileNavOpen = false;
		}
	};

	update();
	mqXs.addEventListener('change', update);
	mqPhablet.addEventListener('change', update);
	mqTablet.addEventListener('change', update);
}

export function getViewport() {
	return {
		get breakpoint() {
			return breakpoint;
		},
		get isMobile() {
			return breakpoint === 'xs';
		},
		/** Any width where the detail panel can no longer dock beside the list. */
		get isCompact() {
			return breakpoint !== 'desktop';
		},
		get mobileNavOpen() {
			return mobileNavOpen;
		},
		openMobileNav() {
			mobileNavOpen = true;
		},
		closeMobileNav() {
			mobileNavOpen = false;
		}
	};
}
