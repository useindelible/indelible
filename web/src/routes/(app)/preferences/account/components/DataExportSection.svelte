<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
</script>

<SettingsGroup title="Export your data">
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">Download your full archive</div>
				<div class="hint">
					A single ZIP with every saved item, highlight, note, and collection. Ready in a few
					minutes — we'll email you when it's done.
				</div>
			</div>
			<div>
				<button type="button" class="btn ghost" disabled>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M12 3v12" />
						<path d="M7 10l5 5 5-5" />
						<path d="M5 21h14" />
					</svg>
					Request archive
				</button>
			</div>
		</div>
	</div>
</SettingsGroup>
