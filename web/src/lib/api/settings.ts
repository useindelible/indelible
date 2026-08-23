import {
	getArchival,
	getNotifications,
	getPreferences,
	updateArchival,
	updateNotifications,
	updatePreferences,
	type ArchivalSettingsResponse,
	type NotificationsSettingsResponse,
	type PreferencesSettingsResponse
} from '$lib/api';
import { get } from 'svelte/store';
import { t } from '$lib/i18n';

type ApiProblem = {
	detail?: string;
	error?: string;
	message?: string;
};

type ApiResult<T> = { success: true; data: T } | { success: false; error: string };

function extractMessage(problem: unknown, fallback: string): string {
	if (!problem || typeof problem !== 'object') {
		return fallback;
	}

	const candidate = problem as ApiProblem;
	return candidate.detail ?? candidate.message ?? candidate.error ?? fallback;
}

export async function loadPreferencesSettings(): Promise<ApiResult<PreferencesSettingsResponse>> {
	try {
		const { data, error } = await getPreferences();
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to load preferences') };
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}

export async function savePreferencesSettings(
	body: PreferencesSettingsResponse
): Promise<ApiResult<PreferencesSettingsResponse>> {
	try {
		const { data, error } = await updatePreferences({ body });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to save preferences') };
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}

export async function loadNotificationsSettings(): Promise<
	ApiResult<NotificationsSettingsResponse>
> {
	try {
		const { data, error } = await getNotifications();
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to load notifications') };
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}

export async function saveNotificationsSettings(
	body: NotificationsSettingsResponse
): Promise<ApiResult<NotificationsSettingsResponse>> {
	try {
		const { data, error } = await updateNotifications({ body });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to save notifications') };
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}

export async function loadArchivalSettings(): Promise<ApiResult<ArchivalSettingsResponse>> {
	try {
		const { data, error } = await getArchival();
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to load archival settings') };
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}

export async function saveArchivalSettings(
	body: ArchivalSettingsResponse
): Promise<ApiResult<ArchivalSettingsResponse>> {
	try {
		const { data, error } = await updateArchival({ body });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to save archival settings') };
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}
