import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

import IdentitySection from '../../src/routes/(app)/preferences/account/components/IdentitySection.svelte';

describe('IdentitySection', () => {
	it('offers no bio control, since the profile contract cannot store one', () => {
		render(IdentitySection, {
			props: {
				displayName: 'Sam',
				username: '@sam',
				onDisplayNameChange: vi.fn()
			}
		});

		expect(screen.getByLabelText('Display name')).toBeTruthy();
		expect(screen.queryByLabelText('Bio')).toBeNull();
		expect(screen.queryByText('Bio')).toBeNull();
	});
});
