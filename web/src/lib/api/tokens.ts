import {
	createToken,
	listTokens,
	revokeToken,
	type ApiTokenResponse,
	type CreateApiTokenRequest,
	type CreateApiTokenResponse
} from '$lib/api';

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

export async function loadApiTokens(): Promise<ApiResult<ApiTokenResponse[]>> {
	const { data, error } = await listTokens();
	if (data) {
		return { success: true, data: data.data };
	}
	return { success: false, error: extractMessage(error, 'Failed to load API tokens') };
}

export async function createApiToken(
	body: CreateApiTokenRequest
): Promise<ApiResult<CreateApiTokenResponse>> {
	const { data, error } = await createToken({ body });
	if (data) {
		return { success: true, data };
	}
	return { success: false, error: extractMessage(error, 'Failed to create token') };
}

export async function revokeApiToken(tokenId: string): Promise<ApiResult<null>> {
	const { error } = await revokeToken({ path: { token_id: tokenId } });
	if (!error) {
		return { success: true, data: null };
	}
	return { success: false, error: extractMessage(error, 'Failed to revoke token') };
}
