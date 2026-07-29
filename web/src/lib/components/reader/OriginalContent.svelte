<script lang="ts">
	interface Props {
		downloadUrl: string;
	}

	let { downloadUrl }: Props = $props();
</script>

<div class="original-container">
	<iframe
		src={downloadUrl}
		title="Original page"
		sandbox="allow-same-origin"
		class="original-iframe"
	></iframe>
</div>

<style>
	.original-container {
		flex: 1;
		overflow: hidden;
	}

	.original-iframe {
		width: 100%;
		height: 100%;
		border: none;
	}
</style>
