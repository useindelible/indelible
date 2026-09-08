export type ModalType = 'url' | 'upload' | 'email' | 'rss' | 'x' | 'youtube';

let activeModal = $state<ModalType | null>(null);
let popoverOpen = $state(false);
let subscribedCount = $state(0);

export function getModalStore() {
	return {
		get active() {
			return activeModal;
		},
		open(type: ModalType) {
			popoverOpen = false;
			activeModal = type;
		},
		close() {
			activeModal = null;
		},
		get popoverOpen() {
			return popoverOpen;
		},
		/**
		 * True while any add surface owns the keyboard. The popover's trigger keeps
		 * focus outside the menu, so callers cannot infer this from an event target.
		 */
		get overlayOpen() {
			return activeModal !== null || popoverOpen;
		},
		togglePopover() {
			popoverOpen = !popoverOpen;
		},
		closePopover() {
			popoverOpen = false;
		},
		get subscribedCount() {
			return subscribedCount;
		},
		notifySubscribed() {
			subscribedCount++;
		}
	};
}

// Convenience alias used by the layout keyboard shortcut (Cmd+N → URL modal).
export function getAddItemModal() {
	const store = getModalStore();
	return {
		show() {
			store.open('url');
		}
	};
}
