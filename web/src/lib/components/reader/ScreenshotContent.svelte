<script lang="ts">
	interface Props {
		downloadUrl: string;
	}

	let { downloadUrl }: Props = $props();
</script>

<div class="screenshot-container">
	<img src={downloadUrl} alt="Page screenshot" class="screenshot-img" />
</div>

<style>
	.screenshot-container {
		flex: 1;
		overflow: auto;
		padding: 32px;
		display: flex;
		justify-content: center;
	}

	.screenshot-container::-webkit-scrollbar {
		width: 6px;
	}

	.screenshot-container::-webkit-scrollbar-track {
		background: transparent;
	}

	.screenshot-container::-webkit-scrollbar-thumb {
		background: var(--text-quaternary);
		border-radius: 3px;
	}

	.screenshot-img {
		max-width: 100%;
		height: auto;
		border-radius: 8px;
		box-shadow: var(--shadow-1);
	}
</style>
