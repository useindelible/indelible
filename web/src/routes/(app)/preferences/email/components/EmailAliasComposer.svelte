<script lang="ts">
	import type { AliasDestinationDto } from '$lib/api';
	import { domainFromAddress, isValidLocalPart } from '../email-model';

	interface Props {
		open: boolean;
		destination: AliasDestinationDto;
		address: string;
		localPart: string;
		creating: boolean;
		error: string | null;
		onDestination: (destination: AliasDestinationDto) => void;
		onLocalPart: (value: string) => void;
		onClose: () => void;
		onCreate: () => void;
	}

	let {
		open,
		destination,
		address,
		localPart,
		creating,
		error,
		onDestination,
		onLocalPart,
		onClose,
		onCreate
	}: Props = $props();

	const destinationLabel = $derived(destination === 'feed' ? 'Feed' : 'Library');
</script>

<div class="draft-shell">
	<div class="draft-composer" class:open aria-hidden={!open}>
		<div class="draft-inner">
			<div class="draft-head">
				<div>
					<div class="draft-eyebrow">Replace your {destinationLabel} address</div>
					<div class="draft-title">Issue a new primary <em>{destinationLabel}</em> address.</div>
				</div>
				<button class="draft-close" type="button" aria-label="Close composer" onclick={onClose}>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M18 6L6 18M6 6l12 12" />
					</svg>
				</button>
			</div>

			<div class="seg-toggle" role="tablist" aria-label="Destination">
				<button
					class="seg-btn feed"
					class:active={destination === 'feed'}
					type="button"
					role="tab"
					aria-selected={destination === 'feed'}
					onclick={() => onDestination('feed')}
				>
					<span class="seg-dot" aria-hidden="true"></span>
					Feed
				</button>
				<button
					class="seg-btn library"
					class:active={destination === 'library'}
					type="button"
					role="tab"
					aria-selected={destination === 'library'}
					onclick={() => onDestination('library')}
				>
					<span class="seg-dot" aria-hidden="true"></span>
					Library
				</button>
			</div>

			<div class="draft-line">
				<input
					type="text"
					class="draft-input"
					placeholder="local-part"
					value={localPart}
					maxlength="32"
					autocomplete="off"
					autocorrect="off"
					autocapitalize="off"
					spellcheck="false"
					oninput={(event) => onLocalPart(event.currentTarget.value)}
				/>
				<span class="draft-suffix">{domainFromAddress(address)}</span>
			</div>

			<p class="draft-warning">
				Your current <span class="draft-warning-dest">{destination}</span> address keeps receiving for
				28 days, then stops.
			</p>

			<div class="draft-foot">
				<div class="draft-hint">
					Lowercase letters, digits, <code>. _ -</code> · 3–32 characters
				</div>
				<div class="draft-actions">
					<button class="btn-ghost" type="button" onclick={onClose}>Cancel</button>
					<button
						class="btn-seal"
						type="button"
						disabled={creating || !isValidLocalPart(localPart)}
						onclick={onCreate}
					>
						<span class="seal" aria-hidden="true">i</span>
						<span class="seal-label">{creating ? 'Creating…' : 'Make it primary'}</span>
					</button>
				</div>
			</div>

			{#if error}
				<p class="form-error">{error}</p>
			{/if}
		</div>
	</div>
</div>

<style>
	.draft-shell {
		margin-top: 18px;
	}

	.draft-composer {
		position: relative;
		background: var(--paper);
		border-radius: 6px;
		box-shadow: var(--envelope-shadow);
		overflow: hidden;
		max-height: 0;
		opacity: 0;
		transform: translateY(-4px);
		pointer-events: none;
		transition:
			max-height 360ms cubic-bezier(0.2, 0, 0, 1),
			opacity 240ms ease,
			transform 240ms ease;
	}

	.draft-composer.open {
		max-height: 520px;
		opacity: 1;
		transform: translateY(0);
		pointer-events: auto;
	}

	.draft-composer::before {
		content: '';
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 4px;
		background: linear-gradient(180deg, var(--accent), var(--accent-strong));
		z-index: 2;
	}

	.draft-composer::after {
		content: '';
		position: absolute;
		inset: 0;
		pointer-events: none;
		background-image: linear-gradient(
			transparent 27px,
			var(--border-hairline) 28px,
			transparent 29px
		);
		background-size: 100% 28px;
		background-position: 0 32px;
		opacity: 0.55;
		z-index: 0;
	}

	.draft-inner {
		padding: 26px 30px 22px 34px;
		position: relative;
		z-index: 1;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.draft-head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
	}

	.draft-eyebrow {
		font-family: var(--font-mono);
		font-size: 9.5px;
		font-weight: 600;
		letter-spacing: 0.22em;
		text-transform: uppercase;
		color: var(--accent-strong);
		margin-bottom: 5px;
	}

	.draft-title {
		font-family: var(--font-display);
		font-style: italic;
		font-size: 20px;
		font-weight: 500;
		letter-spacing: -0.018em;
		color: var(--text-primary);
		line-height: 1.15;
	}

	.draft-title em {
		font-style: italic;
		color: var(--accent);
	}

	.draft-close {
		width: 24px;
		height: 24px;
		border-radius: var(--radius-circle);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.draft-close:hover {
		background: var(--fill-hover);
	}

	.draft-close svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.seg-toggle {
		display: inline-flex;
		align-items: center;
		padding: 3px;
		border-radius: 980px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		align-self: flex-start;
	}

	.seg-btn {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		border: none;
		border-radius: 980px;
		background: transparent;
		color: var(--text-secondary);
		padding: 6px 14px;
		cursor: pointer;
		font-family: var(--font-body);
		font-size: 12.5px;
		font-weight: 500;
		letter-spacing: -0.005em;
		transition:
			background 140ms ease,
			color 140ms ease,
			box-shadow 140ms ease;
	}

	.seg-btn:hover {
		color: var(--text-primary);
	}

	.seg-dot {
		width: 7px;
		height: 7px;
		border-radius: var(--radius-circle);
		background: var(--accent);
	}

	.seg-btn.library .seg-dot {
		background: var(--airmail-navy);
	}

	.seg-btn.active {
		background: var(--paper);
		color: var(--text-primary);
		font-weight: 600;
		box-shadow:
			0 1px 3px rgba(26, 22, 18, 0.08),
			0 0 0 0.5px var(--border-primary);
	}

	.draft-line {
		display: flex;
		align-items: stretch;
		gap: 0;
		border-radius: 8px;
		background: var(--paper-soft);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		overflow: hidden;
		font-family: var(--font-mono);
		font-size: 14px;
		color: var(--text-primary);
		transition:
			box-shadow 160ms ease,
			background 160ms ease;
	}

	.draft-line:focus-within {
		background: var(--paper);
		box-shadow:
			inset 0 0 0 1.5px var(--accent),
			0 0 0 4px var(--accent-soft);
	}

	.draft-input {
		flex: 1;
		min-width: 0;
		padding: 12px 16px;
		background: transparent;
		border: none;
		outline: none;
		font: inherit;
		font-weight: 500;
		color: inherit;
		letter-spacing: -0.005em;
	}

	.draft-input::placeholder {
		color: var(--text-tertiary);
	}

	.draft-suffix {
		display: inline-flex;
		align-items: center;
		padding: 0 16px;
		color: var(--text-tertiary);
		background: var(--paper-deep);
		border-left: 0.5px solid var(--border-primary);
		letter-spacing: -0.005em;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.draft-warning,
	.draft-hint,
	.form-error {
		font-size: 12px;
		color: var(--text-secondary);
		margin: 0;
	}

	.draft-warning {
		font-family: var(--font-body);
		line-height: 1.5;
		letter-spacing: -0.005em;
		padding: 10px 12px 10px 14px;
		border-left: 2px solid var(--warning);
		background: var(--warning-soft);
		border-radius: 0 6px 6px 0;
	}

	.draft-warning-dest {
		font-family: var(--font-mono);
		font-size: 11.5px;
		color: var(--text-primary);
	}

	.draft-foot {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding-top: 16px;
		border-top: 0.5px dashed var(--border-primary);
	}

	.draft-hint {
		font-size: 11.5px;
		color: var(--text-tertiary);
		line-height: 1.4;
		flex: 1;
		min-width: 0;
		max-width: 360px;
	}

	.draft-hint code {
		font-family: var(--font-mono);
		font-size: 11px;
		padding: 1px 4px;
		border-radius: 3px;
		background: var(--bg-secondary);
		color: var(--text-secondary);
	}

	.draft-actions {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
	}

	.btn-ghost {
		border: none;
		border-radius: var(--radius-sm);
		padding: 6px 12px;
		background: transparent;
		color: var(--text-secondary);
		font-size: 12.5px;
		font-weight: 500;
		cursor: pointer;
		letter-spacing: -0.005em;
	}

	.btn-ghost:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.btn-seal {
		position: relative;
		display: inline-flex;
		align-items: center;
		gap: 9px;
		padding: 7px 18px 7px 8px;
		border-radius: var(--radius-full);
		background: var(--ink);
		color: var(--text-on-color);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		border: none;
		letter-spacing: -0.005em;
		box-shadow:
			0 4px 14px rgba(26, 22, 18, 0.18),
			inset 0 0 0 0.5px rgba(255, 255, 255, 0.14);
		transition:
			transform 140ms ease,
			box-shadow 140ms ease,
			opacity 140ms ease;
	}

	.btn-seal:hover:not(:disabled) {
		transform: translateY(-1px);
		box-shadow:
			0 8px 20px rgba(26, 22, 18, 0.26),
			inset 0 0 0 0.5px rgba(255, 255, 255, 0.18);
	}

	.btn-seal:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.seal {
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: var(--radius-circle);
		background:
			radial-gradient(circle at 35% 30%, rgba(255, 255, 255, 0.22), transparent 55%),
			rgba(255, 255, 255, 0.1);
		color: var(--text-on-color);
		font-family: var(--font-display);
		font-style: italic;
		font-weight: 700;
		font-size: 13px;
		letter-spacing: -0.04em;
		box-shadow:
			inset 0 0 0 1px rgba(255, 255, 255, 0.3),
			inset 0 0 0 4px rgba(255, 255, 255, 0.1);
	}

	.form-error {
		color: var(--destructive);
	}
</style>
