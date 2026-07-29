import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	resolve: {
		conditions: ['browser']
	},
	test: {
		include: ['tests/**/*.test.ts', 'src/**/*.test.ts', 'src/**/*.test.svelte.ts'],
		environment: 'jsdom',
		setupFiles: ['./vitest-setup.ts']
	}
});
