<script lang="ts">
	interface Props {
		message: string;
	}

	let { message }: Props = $props();
</script>

<div class="notice" role="alert">{message}</div>

<style>
	.notice {
		position: fixed;
		top: 16px;
		left: 50%;
		z-index: 1000;
		max-width: min(560px, calc(100vw - 32px));
		transform: translateX(-50%);
		padding: 10px 14px;
		border: 1px solid var(--destructive);
		border-radius: 10px;
		background: var(--bg-elevated);
		color: var(--text-primary);
		box-shadow: var(--shadow-3);
		font-size: 13px;
	}
</style>
