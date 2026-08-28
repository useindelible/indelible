class MockPage {
	params = $state<Record<string, string>>({ slug: 'ent_a' });
	url = $state({ pathname: '/entities/ent_a' });
}

export const page = new MockPage();
