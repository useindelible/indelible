type ProtectedRouteRedirectInput = {
	pathname: string;
	isAuthenticated: boolean;
	needsOnboarding: boolean;
	setupRequired?: boolean;
};

type PublicRouteRedirectInput = {
	pathname: string;
	isAuthenticated: boolean;
	needsOnboarding: boolean;
	needsVerification: boolean;
	setupRequired?: boolean;
};

export function getProtectedRouteRedirect({
	pathname,
	isAuthenticated,
	needsOnboarding,
	setupRequired = false
}: ProtectedRouteRedirectInput): string | null {
	if (!isAuthenticated) {
		return setupRequired ? '/register' : '/login';
	}

	const isOnboardingRoute = pathname.startsWith('/onboarding');
	if (needsOnboarding && !isOnboardingRoute) {
		return '/onboarding/welcome';
	}

	if (!needsOnboarding && isOnboardingRoute) {
		return '/';
	}

	return null;
}

export function getPublicRouteRedirect({
	pathname,
	isAuthenticated,
	needsOnboarding,
	needsVerification,
	setupRequired = false
}: PublicRouteRedirectInput): string | null {
	if (!isAuthenticated) {
		if (setupRequired) {
			return pathname === '/register' ? null : '/register';
		}
		return null;
	}

	if (needsVerification && pathname === '/verify-email') {
		return null;
	}

	if (needsOnboarding) {
		return '/onboarding/welcome';
	}

	return '/';
}
