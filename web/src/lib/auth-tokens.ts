let currentAccessToken: string | null = null;

export function getAccessToken(): string | null {
	return currentAccessToken;
}

export function setAccessToken(token: string | null): void {
	currentAccessToken = token;
}
