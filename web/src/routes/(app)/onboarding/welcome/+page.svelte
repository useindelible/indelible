<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import Button from '$lib/components/ui/Button.svelte';
	import { t } from '$lib/i18n';
</script>

<div class="welcome">
	<svg class="backdrop" viewBox="0 0 600 520" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
		<defs>
			<pattern id="dot-grid" x="0" y="0" width="24" height="24" patternUnits="userSpaceOnUse">
				<circle cx="1" cy="1" r="1" fill="#0071E3" fill-opacity="0.07" />
			</pattern>
		</defs>

		<!-- Dot grid -->
		<rect width="600" height="520" fill="url(#dot-grid)" />

		<!-- Article card outlines — top-left cluster, slightly rotated -->
		<g opacity="0.055" stroke="#0071E3" stroke-width="1" fill="none">
			<rect x="28" y="32" width="110" height="148" rx="6" transform="rotate(-6 83 106)" />
			<rect x="48" y="44" width="110" height="148" rx="6" transform="rotate(-3 103 118)" />
			<!-- Text line suggestions on front card -->
			<line x1="58" y1="80" x2="140" y2="80" stroke-linecap="round" transform="rotate(-3 99 80)" />
			<line x1="58" y1="92" x2="130" y2="92" stroke-linecap="round" transform="rotate(-3 94 92)" />
			<line
				x1="58"
				y1="104"
				x2="120"
				y2="104"
				stroke-linecap="round"
				transform="rotate(-3 89 104)"
			/>
		</g>

		<!-- Article card outlines — top-right cluster -->
		<g opacity="0.055" stroke="#0071E3" stroke-width="1" fill="none">
			<rect x="462" y="28" width="110" height="148" rx="6" transform="rotate(5 517 102)" />
			<rect x="444" y="42" width="110" height="148" rx="6" transform="rotate(2 499 116)" />
			<line x1="454" y1="82" x2="536" y2="82" stroke-linecap="round" transform="rotate(2 495 82)" />
			<line x1="454" y1="94" x2="526" y2="94" stroke-linecap="round" transform="rotate(2 490 94)" />
			<line
				x1="454"
				y1="106"
				x2="516"
				y2="106"
				stroke-linecap="round"
				transform="rotate(2 485 106)"
			/>
		</g>

		<!-- Lone card bottom-left -->
		<g opacity="0.04" stroke="#0071E3" stroke-width="1" fill="none">
			<rect x="18" y="360" width="90" height="120" rx="5" transform="rotate(-8 63 420)" />
			<line
				x1="30"
				y1="390"
				x2="96"
				y2="390"
				stroke-linecap="round"
				transform="rotate(-8 63 390)"
			/>
			<line
				x1="30"
				y1="402"
				x2="88"
				y2="402"
				stroke-linecap="round"
				transform="rotate(-8 59 402)"
			/>
		</g>

		<!-- Lone card bottom-right -->
		<g opacity="0.04" stroke="#0071E3" stroke-width="1" fill="none">
			<rect x="492" y="370" width="90" height="120" rx="5" transform="rotate(6 537 430)" />
			<line
				x1="504"
				y1="400"
				x2="570"
				y2="400"
				stroke-linecap="round"
				transform="rotate(6 537 400)"
			/>
			<line
				x1="504"
				y1="412"
				x2="562"
				y2="412"
				stroke-linecap="round"
				transform="rotate(6 533 412)"
			/>
		</g>
	</svg>

	<div class="logo-mark" aria-hidden="true">
		<svg width="32" height="32" viewBox="0 0 32 32" fill="none">
			<path
				d="M8 6h10l6 6v14a2 2 0 01-2 2H8a2 2 0 01-2-2V8a2 2 0 012-2z"
				stroke="white"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
			<path
				d="M18 6v6h6"
				stroke="white"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
			<path d="M10 18h12M10 22h8" stroke="white" stroke-width="1.5" stroke-linecap="round" />
		</svg>
	</div>

	<p class="welcome-to">{$t('onboarding_welcome_to')}</p>
	<p class="welcome-brand">indelible</p>

	<ul class="feature-list">
		<li class="feature-item">
			<div class="feature-icon feature-icon--blue" aria-hidden="true">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none">
					<circle cx="12" cy="12" r="9" stroke="#0071E3" stroke-width="1.5" />
					<path d="M12 3v18M3 12h18" stroke="#0071E3" stroke-width="1.5" stroke-linecap="round" />
				</svg>
			</div>
			<span>{$t('onboarding_welcome_save_web')}</span>
		</li>
		<li class="feature-item">
			<div class="feature-icon feature-icon--amber" aria-hidden="true">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none">
					<path
						d="M12 2a7 7 0 017 7c0 3-2 5.5-4 7.5L12 20l-3-3.5C7 14.5 5 12 5 9a7 7 0 017-7z"
						stroke="#D97706"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
					<circle cx="12" cy="9" r="2" stroke="#D97706" stroke-width="1.5" />
				</svg>
			</div>
			<span>{$t('onboarding_welcome_mila')}</span>
		</li>
		<li class="feature-item">
			<div class="feature-icon feature-icon--emerald" aria-hidden="true">
				<svg
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#059669"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71" />
					<path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71" />
				</svg>
			</div>
			<span>{$t('onboarding_welcome_sync')}</span>
		</li>
	</ul>

	<div class="cta-wrapper">
		<Button
			variant="primary"
			size="xl"
			fullWidth
			onclick={() => goto(resolve('/onboarding/account'))}
		>
			{$t('onboarding_get_started')}
		</Button>
	</div>
</div>

<style>
	.welcome {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 100%;
		max-width: 600px;
		border-radius: 14px;
		padding: 48px 40px 40px;
		overflow: hidden;
		background: var(--bg-elevated);
		box-shadow:
			0 1px 3px rgba(0, 0, 0, 0.08),
			0 0 0 1px rgba(0, 0, 0, 0.07);
	}

	:global([data-theme='dark']) .welcome {
		background: #161614;
		box-shadow:
			0 1px 3px rgba(0, 0, 0, 0.4),
			0 0 0 1px rgba(255, 255, 255, 0.07);
	}

	.backdrop {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
		user-select: none;
	}

	:global([data-theme='dark']) .backdrop circle {
		fill: #0071e3;
		fill-opacity: 0.1;
	}

	.logo-mark {
		position: relative;
		width: 56px;
		height: 56px;
		border-radius: 14px;
		background: #0071e3;
		box-shadow: 0 2px 12px rgba(0, 113, 227, 0.28);
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 24px;
		flex-shrink: 0;
	}

	:global([data-theme='dark']) .logo-mark {
		background: #0a84ff;
		box-shadow: 0 2px 12px rgba(10, 132, 255, 0.35);
	}

	.welcome-to {
		position: relative;
		font-family: var(--font-sans);
		font-size: 30px;
		font-weight: 600;
		letter-spacing: -0.03em;
		line-height: 1.15;
		color: var(--text-secondary);
		margin: 0;
		text-align: center;
	}

	.welcome-brand {
		position: relative;
		font-family: var(--font-sans);
		font-size: 30px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.15;
		color: #0071e3;
		margin: 0 0 36px;
		text-align: center;
	}

	:global([data-theme='dark']) .welcome-brand {
		color: #0a84ff;
	}

	.feature-list {
		position: relative;
		list-style: none;
		padding: 0;
		margin: 0 0 36px;
		width: 100%;
		display: flex;
		flex-direction: column;
		gap: 0;
		border: 1px solid rgba(0, 0, 0, 0.07);
		border-radius: 10px;
		overflow: hidden;
	}

	:global([data-theme='dark']) .feature-list {
		border-color: rgba(255, 255, 255, 0.07);
	}

	.feature-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 14px 16px;
		background: rgba(255, 255, 255, 0.8);
	}

	.feature-item + .feature-item {
		border-top: 1px solid rgba(0, 0, 0, 0.06);
	}

	:global([data-theme='dark']) .feature-item {
		background: rgba(255, 255, 255, 0.03);
	}

	:global([data-theme='dark']) .feature-item + .feature-item {
		border-color: rgba(255, 255, 255, 0.06);
	}

	.feature-icon {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.feature-icon--blue {
		background: rgba(0, 113, 227, 0.08);
	}
	.feature-icon--amber {
		background: rgba(217, 119, 6, 0.08);
	}
	.feature-icon--emerald {
		background: rgba(5, 150, 105, 0.08);
	}

	:global([data-theme='dark']) .feature-icon--blue {
		background: rgba(10, 132, 255, 0.14);
	}
	:global([data-theme='dark']) .feature-icon--amber {
		background: rgba(245, 158, 11, 0.14);
	}
	:global([data-theme='dark']) .feature-icon--emerald {
		background: rgba(16, 185, 129, 0.14);
	}

	.feature-item span {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-primary);
	}

	.cta-wrapper {
		position: relative;
		width: 100%;
		max-width: 320px;
	}
</style>
