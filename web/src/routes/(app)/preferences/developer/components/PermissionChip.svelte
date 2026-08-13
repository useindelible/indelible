<script lang="ts">
	import { permissionClass } from '../developer-model';

	interface Props {
		permission: string;
	}

	let { permission }: Props = $props();
</script>

<span class="permission {permissionClass(permission)}">{permission}</span>

<style>
	.permission {
		display: inline-flex;
		align-items: center;
		padding: 2px 7px;
		border-radius: 5px;
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: -0.01em;
		text-transform: lowercase;
	}

	.permission.read {
		background: var(--dev-scope-read-bg);
		color: var(--dev-scope-read-fg);
	}

	.permission.write {
		background: var(--dev-scope-write-bg);
		color: var(--dev-scope-write-fg);
	}

	.permission.other {
		background: var(--dev-scope-other-bg);
		color: var(--dev-scope-other-fg);
	}

	.permission.obsidian {
		background: var(--dev-scope-ext-bg);
		color: var(--dev-scope-ext-fg);
	}
</style>
