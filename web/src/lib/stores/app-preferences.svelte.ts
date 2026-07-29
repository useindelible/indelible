import type { DefaultViewDto } from '$lib/api';
import { loadPreferencesSettings } from '$lib/api/settings';

function createAppPreferences() {
	let defaultView = $state<DefaultViewDto>('library');
	let loaded = false;

	async function load() {
		if (loaded) return;
		loaded = true;
		const result = await loadPreferencesSettings();
		if (result.success) {
			defaultView = result.data.layout.default_view;
		}
	}

	return {
		get defaultView() {
			return defaultView;
		},
		setDefaultView(v: DefaultViewDto) {
			defaultView = v;
		},
		load
	};
}

let instance: ReturnType<typeof createAppPreferences> | null = null;

export function getAppPreferences() {
	if (!instance) {
		instance = createAppPreferences();
	}
	return instance;
}
