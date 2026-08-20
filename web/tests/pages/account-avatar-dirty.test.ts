import { fireEvent, render, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const auth = vi.hoisted(() => ({
	user: {
		avatar_url: 'https://example.test/original.png',
		created_at: '2026-08-01T00:00:00Z',
		display_name: 'Sama',
		email: 'sam@example.test',
		email_verified: true
	},
	updateProfile: vi.fn()
}));

vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => auth
}));

vi.mock('$lib/api/avatar', () => ({
	MAX_AVATAR_SIZE_BYTES: 2 * 1024 * 1024,
	uploadAvatar: vi.fn()
}));

import AccountPage from '../../src/routes/(app)/preferences/account/+page.svelte';

describe('account avatar changes', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.stubGlobal('URL', {
			...URL,
			createObjectURL: vi.fn(() => 'blob:test-avatar'),
			revokeObjectURL: vi.fn()
		});
	});

	it('shows the save pill when a valid avatar is selected', async () => {
		const { container } = render(AccountPage);
		const input = container.querySelector<HTMLInputElement>('input[type="file"]');
		expect(input).toBeTruthy();

		const avatar = new File(['avatar'], 'avatar.png', { type: 'image/png' });
		await fireEvent.change(input!, { target: { files: [avatar] } });

		await waitFor(() => {
			expect(container.querySelector('.save-pill')?.classList.contains('visible')).toBe(true);
		});
	});
});
