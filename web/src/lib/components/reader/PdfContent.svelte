<script lang="ts">
	interface Props {
		downloadUrl: string;
	}

	let { downloadUrl }: Props = $props();
</script>

<div class="pdf-container">
	<embed src={downloadUrl} type="application/pdf" class="pdf-embed" title="PDF document" />
	<p class="pdf-fallback">
		If the PDF does not load, you can
		<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external presigned S3 URL, not an app route -->
		<a href={downloadUrl} target="_blank" rel="noopener">download it directly</a>.
	</p>
</div>

<style>
	.pdf-container {
		flex: 1;
		overflow: hidden;
		display: flex;
	}

	.pdf-embed {
		width: 100%;
		height: 100%;
		border: none;
	}

	.pdf-fallback {
		padding: 40px;
		text-align: center;
		color: var(--text-secondary);
		font-size: 14px;
		font-family: var(--font-sans);
	}

	.pdf-fallback a {
		color: var(--accent);
		text-decoration: none;
	}

	.pdf-fallback a:hover {
		text-decoration: underline;
	}
</style>
