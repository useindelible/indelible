<script lang="ts">
	import TtsBanner from './TtsBanner.svelte';
	import { formatTtsResumePosition } from './tts-controller-model';
	import { t } from '$lib/i18n';

	interface Props {
		positionSeconds: number;
		onResume: () => void;
		onStartAgain: () => void;
	}

	let { positionSeconds, onResume, onStartAgain }: Props = $props();

	const message = $derived(
		$t('reader_tts_resume_position', {
			values: { position: formatTtsResumePosition(positionSeconds) }
		})
	);
</script>

<TtsBanner
	variant="setup"
	title={$t('reader_tts_resume_title')}
	{message}
	actions={[
		{
			label: $t('reader_resume'),
			style: 'primary',
			onclick: onResume
		},
		{
			label: $t('reader_tts_start_again'),
			style: 'secondary',
			onclick: onStartAgain
		}
	]}
/>
