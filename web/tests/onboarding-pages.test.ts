import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { createApiModuleMock } from './helpers/api-module-mock';

const mockGoto = vi.fn();
vi.mock('$app/navigation', () => ({
	goto: (...args: unknown[]) => mockGoto(...args)
}));

vi.mock('$app/state', () => ({
	page: { data: { stepIndex: 0, firstIncompletePath: 'welcome' } }
}));

vi.mock('$app/paths', () => ({
	resolve: (path: string) => path
}));

vi.mock('$lib/api', () => createApiModuleMock());

import WelcomePage from '../src/routes/(app)/onboarding/welcome/+page.svelte';
import AccountPage from '../src/routes/(app)/onboarding/account/+page.svelte';
import AddContentPage from '../src/routes/(app)/onboarding/add-content/+page.svelte';
import FeedsPage from '../src/routes/(app)/onboarding/feeds/+page.svelte';
import AiPage from '../src/routes/(app)/onboarding/ai/+page.svelte';
import ReadyPage from '../src/routes/(app)/onboarding/ready/+page.svelte';
import { completeStep, subscribe, testConfig } from '$lib/api';

const mockCompleteStep = vi.mocked(completeStep);
const mockSubscribe = vi.mocked(subscribe);
const mockTestConfig = vi.mocked(testConfig);

describe('Welcome page (step 1)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders hero text', () => {
		render(WelcomePage);
		expect(screen.getByText('Welcome to')).toBeTruthy();
		expect(screen.getByText('indelible')).toBeTruthy();
	});

	it('renders Get Started button', () => {
		render(WelcomePage);
		expect(screen.getByText('Get Started')).toBeTruthy();
	});

	it('renders three value proposition items', () => {
		render(WelcomePage);
		expect(screen.getByText('Save anything from the web')).toBeTruthy();
		expect(screen.getByText('AI-powered reading with Mila')).toBeTruthy();
		expect(screen.getByText('Sync highlights to Obsidian & Notion')).toBeTruthy();
	});

	it('Get Started navigates to account step', async () => {
		render(WelcomePage);
		const button = screen.getByText('Get Started');
		button.click();
		expect(mockGoto).toHaveBeenCalledWith('/onboarding/account');
	});
});

describe('Account page (step 2)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders step title', () => {
		render(AccountPage);
		expect(screen.getByText('Set up your profile')).toBeTruthy();
	});

	it('renders display name input', () => {
		render(AccountPage);
		const input = document.querySelector('input[type="text"]') as HTMLInputElement;
		expect(input).toBeTruthy();
	});

	it('renders theme selector cards', () => {
		render(AccountPage);
		expect(screen.getByText('Light')).toBeTruthy();
		expect(screen.getByText('Dark')).toBeTruthy();
		expect(screen.getByText('Auto')).toBeTruthy();
	});

	it('renders Continue button', () => {
		render(AccountPage);
		expect(screen.getByText('Continue')).toBeTruthy();
	});
});

describe('Add Content page (step 3)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders step title', () => {
		render(AddContentPage);
		expect(screen.getByText('Save your first article')).toBeTruthy();
	});

	it('renders both email forwarding options', () => {
		render(AddContentPage);
		expect(screen.getByText('Feed email')).toBeTruthy();
		expect(screen.getByText('Library email')).toBeTruthy();
	});

	it('renders Skip button', () => {
		render(AddContentPage);
		expect(screen.getByText('Skip')).toBeTruthy();
	});
});

describe('Feeds page (step 4)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockCompleteStep.mockResolvedValue({
			data: { current_step: 4, completed: false, steps: [] }
		} as never);
		mockSubscribe.mockResolvedValue({
			data: { is_new: true, subscription: {} }
		} as never);
	});

	it('renders step title', () => {
		render(FeedsPage);
		expect(screen.getByText('Subscribe to your favorite sources')).toBeTruthy();
	});

	it('renders suggested feeds', () => {
		render(FeedsPage);
		expect(screen.getByText('Hacker News')).toBeTruthy();
		expect(screen.getByText('Ars Technica')).toBeTruthy();
	});

	it('leaves every suggested feed unselected by default', () => {
		render(FeedsPage);

		const suggestions = screen.getAllByRole('checkbox', { name: /^Subscribe to / });
		expect(suggestions).toHaveLength(7);
		for (const suggestion of suggestions) {
			expect((suggestion as HTMLInputElement).checked).toBe(false);
		}
		expect(screen.getByRole('status').textContent).toContain('No suggested feeds selected.');
	});

	it('continues with no suggested feeds when none are selected', async () => {
		render(FeedsPage);

		await fireEvent.click(screen.getByRole('button', { name: 'Continue' }));

		await waitFor(() =>
			expect(mockCompleteStep).toHaveBeenCalledWith({
				path: { step: 3 },
				body: { data: { feed_urls: [] } }
			})
		);
		expect(mockGoto).toHaveBeenCalledWith('/onboarding/ai');
	});

	it('does not submit an unconfirmed manual feed when continuing', async () => {
		render(FeedsPage);
		await fireEvent.input(screen.getByLabelText('RSS feed URL'), {
			target: { value: 'https://example.com/draft-feed.xml' }
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Continue' }));

		await waitFor(() =>
			expect(mockCompleteStep).toHaveBeenCalledWith({
				path: { step: 3 },
				body: { data: { feed_urls: [] } }
			})
		);
		expect(mockSubscribe).not.toHaveBeenCalled();
		expect(screen.getByRole('status').textContent).toContain('No suggested feeds selected.');
	});

	it('summarizes and submits only the selected suggested feed', async () => {
		render(FeedsPage);

		await fireEvent.click(screen.getByRole('checkbox', { name: 'Subscribe to Hacker News' }));
		expect(screen.getByRole('status').textContent).toContain('1 suggested feed selected.');

		await fireEvent.click(screen.getByRole('button', { name: 'Continue' }));

		await waitFor(() =>
			expect(mockCompleteStep).toHaveBeenCalledWith({
				path: { step: 3 },
				body: { data: { feed_urls: ['https://hnrss.org/frontpage'] } }
			})
		);
	});

	it('shows the pending completion state while the request runs', async () => {
		let resolveCompletion: (value: unknown) => void = () => undefined;
		mockCompleteStep.mockImplementation(
			() =>
				new Promise((resolve) => {
					resolveCompletion = resolve;
				}) as never
		);
		render(FeedsPage);

		await fireEvent.click(screen.getByRole('button', { name: 'Continue' }));

		expect(screen.getByRole('status').textContent).toContain('Saving your feed choices…');
		expect((screen.getByRole('button', { name: 'Continue' }) as HTMLButtonElement).disabled).toBe(
			true
		);
		expect((screen.getByRole('button', { name: 'Skip' }) as HTMLButtonElement).disabled).toBe(true);

		resolveCompletion({ data: { current_step: 4, completed: false, steps: [] } });
		await waitFor(() => expect(mockGoto).toHaveBeenCalledWith('/onboarding/ai'));
	});

	it('restores onboarding actions after completion fails', async () => {
		mockCompleteStep.mockResolvedValue({
			error: { detail: 'Could not save feed choices.' },
			response: new Response(null, { status: 500 })
		} as never);
		render(FeedsPage);

		await fireEvent.click(screen.getByRole('button', { name: 'Continue' }));

		await waitFor(() => expect(screen.getByText('Could not save feed choices.')).toBeTruthy());
		expect((screen.getByRole('button', { name: 'Continue' }) as HTMLButtonElement).disabled).toBe(
			false
		);
		expect((screen.getByRole('button', { name: 'Skip' }) as HTMLButtonElement).disabled).toBe(
			false
		);
		expect(screen.queryByText('Saving your feed choices…')).toBeNull();
	});

	it('renders RSS URL input', () => {
		render(FeedsPage);
		const input = document.querySelector(
			'input[placeholder="https://example.com/feed.xml"]'
		) as HTMLInputElement;
		expect(input).toBeTruthy();
	});

	it('renders optional OPML upload', () => {
		render(FeedsPage);
		expect(screen.getByText('Drop an OPML file or choose one')).toBeTruthy();
	});

	it('subscribes a manually entered feed without advancing onboarding', async () => {
		render(FeedsPage);
		await fireEvent.input(screen.getByLabelText('RSS feed URL'), {
			target: { value: 'https://www.transfermarkt.de/rss/news' }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Subscribe' }));

		await waitFor(() => expect(screen.getByText('Subscribed successfully.')).toBeTruthy());
		expect(mockSubscribe).toHaveBeenCalledWith({
			body: { url: 'https://www.transfermarkt.de/rss/news' }
		});
		expect(mockCompleteStep).not.toHaveBeenCalled();
		expect((screen.getByLabelText('RSS feed URL') as HTMLInputElement).value).toBe('');
	});

	it('renders Skip button', () => {
		render(FeedsPage);
		expect(screen.getByText('Skip')).toBeTruthy();
	});
});

describe('AI Setup page (step 5)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockCompleteStep.mockResolvedValue({
			data: { current_step: 4, completed: false, steps: [] }
		} as never);
		mockTestConfig.mockResolvedValue({
			data: {
				success: false,
				chat_model_ok: false,
				embedding_model_ok: false,
				error: 'Connection test failed'
			}
		} as never);
	});

	it('renders step title', () => {
		render(AiPage);
		expect(screen.getByText('Supercharge your reading with Mila')).toBeTruthy();
	});

	it('renders provider cards', () => {
		render(AiPage);
		expect(screen.getByText('Local server')).toBeTruthy();
		expect(screen.getByText('OpenAI')).toBeTruthy();
		expect(screen.queryByText('Anthropic')).toBeNull();
	});

	it('shows API key input when cloud provider is selected', async () => {
		render(AiPage);
		const openaiCard = screen.getByText('OpenAI').closest('button');
		openaiCard?.click();
		await vi.dynamicImportSettled();
		const apiKeyInput = document.querySelector('input[type="password"]') as HTMLInputElement;
		expect(apiKeyInput).toBeTruthy();
	});

	it('marks the OpenAI probe as reasoning capable', async () => {
		mockTestConfig.mockResolvedValue({
			data: {
				success: true,
				embedding_dim: 768,
				chat_model_ok: true,
				embedding_model_ok: true
			}
		} as never);
		render(AiPage);
		await fireEvent.click(screen.getByText('OpenAI').closest('button')!);
		await fireEvent.input(screen.getByLabelText('API key'), {
			target: { value: 'sk-test' }
		});
		await fireEvent.click(screen.getByText('Continue'));

		await waitFor(() =>
			expect(mockTestConfig).toHaveBeenCalledWith({
				body: {
					chat_api_base: 'https://api.openai.com/v1',
					chat_api_key: 'sk-test',
					chat_model: 'gpt-5.4-mini',
					supports_reasoning_effort: true,
					embedding_api_base: 'https://api.openai.com/v1',
					embedding_api_key: 'sk-test',
					embedding_model: 'text-embedding-3-small',
					embedding_dim: 768
				}
			})
		);
	});

	it('shows a docker-reachable endpoint input when Local server is selected', async () => {
		render(AiPage);
		const localServerCard = screen.getByText('Local server').closest('button');
		localServerCard?.click();
		await vi.dynamicImportSettled();
		// Inside the shipped Docker stack, localhost is the API container
		// itself — the default must name the host gateway.
		const endpointInput = document.querySelector(
			'input[placeholder="http://host.docker.internal:11434"]'
		) as HTMLInputElement;
		expect(endpointInput).toBeTruthy();
		expect(endpointInput.value).toBe('http://host.docker.internal:11434');
		expect(screen.getByText(/EGRESS_ALLOW_PRIVATE_TARGETS/)).toBeTruthy();
		expect(screen.getByLabelText('Chat model ID')).toBeTruthy();
		expect(screen.getByLabelText('Embedding model ID')).toBeTruthy();
	});

	it('requires both local model IDs before testing', async () => {
		render(AiPage);
		await fireEvent.click(screen.getByText('Local server').closest('button')!);
		await fireEvent.click(screen.getByText('Continue'));

		expect(screen.getByText('Enter both model IDs exposed by your local server.')).toBeTruthy();
		expect(mockTestConfig).not.toHaveBeenCalled();
	});

	it('shows independent local chat and embedding results', async () => {
		mockTestConfig.mockResolvedValue({
			data: {
				success: false,
				embedding_dim: null,
				chat_model_ok: true,
				embedding_model_ok: false,
				chat_error: null,
				embedding_error: 'No embedding model is loaded',
				error: 'No embedding model is loaded'
			}
		} as never);
		render(AiPage);
		await fireEvent.click(screen.getByText('Local server').closest('button')!);
		await fireEvent.input(screen.getByLabelText('Chat model ID'), {
			target: { value: 'gemma-4-e4b-it' }
		});
		await fireEvent.input(screen.getByLabelText('Embedding model ID'), {
			target: { value: 'text-embedding-nomic-embed-text-v1.5' }
		});
		await fireEvent.click(screen.getByText('Continue'));

		await waitFor(() => {
			expect(screen.getByText('Connected to gemma-4-e4b-it')).toBeTruthy();
			expect(screen.getByText('No embedding model is loaded')).toBeTruthy();
		});
		expect(mockCompleteStep).not.toHaveBeenCalled();
	});

	it('persists exact verified local choices', async () => {
		mockTestConfig.mockResolvedValue({
			data: {
				success: true,
				embedding_dim: 768,
				chat_model_ok: true,
				embedding_model_ok: true
			}
		} as never);
		render(AiPage);
		await fireEvent.click(screen.getByText('Local server').closest('button')!);
		await fireEvent.input(screen.getByLabelText('OpenAI-compatible server URL'), {
			target: { value: 'http://localhost:1234/v1/' }
		});
		await fireEvent.input(screen.getByLabelText('Chat model ID'), {
			target: { value: 'gemma-4-e4b-it' }
		});
		await fireEvent.input(screen.getByLabelText('Embedding model ID'), {
			target: { value: 'text-embedding-nomic-embed-text-v1.5' }
		});
		await fireEvent.click(screen.getByText('Continue'));

		await waitFor(() => expect(mockGoto).toHaveBeenCalledWith('/onboarding/ready'));
		expect(mockTestConfig).toHaveBeenCalledWith({
			body: {
				chat_api_base: 'http://localhost:1234/v1',
				chat_model: 'gemma-4-e4b-it',
				embedding_api_base: 'http://localhost:1234/v1',
				embedding_model: 'text-embedding-nomic-embed-text-v1.5',
				embedding_dim: 768
			}
		});
		expect(mockCompleteStep).toHaveBeenCalledWith({
			path: { step: 4 },
			body: {
				data: {
					chat_provider: 'ollama',
					embedding_provider: 'ollama',
					chat_endpoint: 'http://localhost:1234/v1',
					embedding_endpoint: 'http://localhost:1234/v1',
					chat_model: 'gemma-4-e4b-it',
					embedding_model: 'text-embedding-nomic-embed-text-v1.5',
					embedding_dim: 768
				}
			}
		});
	});

	it('renders reassurance text', () => {
		render(AiPage);
		expect(screen.getByText('AI is optional. All features work without it.')).toBeTruthy();
	});

	it('renders Skip button', () => {
		render(AiPage);
		expect(screen.getByRole('button', { name: 'Skip', exact: true })).toBeTruthy();
	});
});

describe('Ready page (step 6)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders completion heading', () => {
		render(ReadyPage);
		expect(screen.getByText("You're all set!")).toBeTruthy();
	});

	it('renders Go to Library button', () => {
		render(ReadyPage);
		expect(screen.getByText('Go to Library')).toBeTruthy();
	});

	it('renders four tips', () => {
		render(ReadyPage);
		expect(screen.getByText('Keyboard shortcuts')).toBeTruthy();
		expect(screen.getByText('Highlight text')).toBeTruthy();
		expect(screen.getByText('Daily review')).toBeTruthy();
		expect(screen.getByText('Save from anywhere')).toBeTruthy();
	});
});
