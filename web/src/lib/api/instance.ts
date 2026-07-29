import { listProviders } from '$lib/api';

export interface InstanceStatus {
	/** Whether the signup form should be available (config flag OR first-run setup). */
	signupsEnabled: boolean;
	/** True when the instance has no users yet and should route to first-run setup. */
	setupRequired: boolean;
}

/**
 * Fetch the instance signup/setup status. Fails closed so a transient API
 * error never opens registration UI on a locked instance.
 */
export async function getInstanceStatus(): Promise<InstanceStatus> {
	try {
		const { data } = await listProviders();
		return {
			signupsEnabled: data?.signups_enabled === true,
			setupRequired: data?.setup_required === true
		};
	} catch {
		return { signupsEnabled: false, setupRequired: false };
	}
}

/** Kept for tests that reset shared module state between component renders. */
export function resetInstanceStatusCache(): void {
	// Status is no longer memoised.
}
