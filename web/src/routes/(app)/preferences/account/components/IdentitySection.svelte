<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';

	interface Props {
		displayName: string;
		bio: string;
		username: string;
		onDisplayNameChange: (value: string) => void;
		onBioChange: (value: string) => void;
	}

	let { displayName, bio, username, onDisplayNameChange, onBioChange }: Props = $props();
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
		<div class="row with-stack">
			<div class="label-block">
				<div class="label">Bio</div>
				<div class="hint">A short blurb that appears on your profile. Markdown is allowed.</div>
			</div>
			<div class="input-group">
				<textarea
					class="textarea"
					value={bio}
					placeholder="A line or two about yourself…"
					maxlength={240}
					aria-label="Bio"
					oninput={(event) => onBioChange((event.currentTarget as HTMLTextAreaElement).value)}
				></textarea>
				<div class="help">Up to 240 characters.</div>
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
