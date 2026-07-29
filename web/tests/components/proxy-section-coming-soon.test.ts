import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import ProxySection from '../../src/routes/(app)/preferences/archival/components/ProxySection.svelte';

function renderSection(proxyUrl = 'socks5://127.0.0.1:1080', proxyAll = true) {
	return render(ProxySection, { props: { proxyUrl, proxyAll } });
}

describe('ProxySection', () => {
	it('marks the section as coming soon', () => {
		renderSection();

		expect(screen.getByText('Coming soon')).toBeTruthy();
		expect(screen.getByText(/saved proxy settings are not active/i)).toBeTruthy();
	});

	it('claims no routing it does not perform', () => {
		const { container } = renderSection();
		const text = container.textContent ?? '';

		expect(text).not.toMatch(/routed through proxy/i);
	});

	it('shows the stored URL but accepts no edits', () => {
		renderSection();

		const input = screen.getByRole('textbox') as HTMLInputElement;
		expect(input.value).toBe('socks5://127.0.0.1:1080');
		expect(input.disabled).toBe(true);
	});

	it('leaves the dependent toggle inert', () => {
		renderSection();

		const toggle = screen.getByRole('switch', { name: 'Proxy for all requests' });
		expect((toggle as HTMLButtonElement).disabled).toBe(true);
	});
});
