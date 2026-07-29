<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';

	interface Props {
		displayName: string;
		username: string;
		onDisplayNameChange: (value: string) => void;
	}

	let { displayName, username, onDisplayNameChange }: Props = $props();
</script>

<SettingsGroup title="Identity">
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">Display name</div>
				<div class="hint">Shown on shared notes and collaboration invites.</div>
			</div>
			<div class="input-group">
				<input
					class="input"
					type="text"
					value={displayName}
					maxlength={100}
					aria-label="Display name"
					oninput={(event) => onDisplayNameChange((event.currentTarget as HTMLInputElement).value)}
				/>
			</div>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">Username</div>
				<div class="hint">Used in your public profile URL. Locked after first set.</div>
			</div>
			<div class="input-group">
				<input
					class="input mono readonly"
					type="text"
					value={username}
					readonly
					aria-label="Username"
				/>
			</div>
		</div>
	</div>
</SettingsGroup>
