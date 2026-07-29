<script lang="ts">
	import TtsBanner from './TtsBanner.svelte';
	import { formatTtsResumePosition } from './tts-controller-model';

	interface Props {
		positionSeconds: number;
		onResume: () => void;
		onStartAgain: () => void;
	}

	let { positionSeconds, onResume, onStartAgain }: Props = $props();

	const message = $derived(`You stopped at ${formatTtsResumePosition(positionSeconds)}.`);
</script>

<TtsBanner
	variant="setup"
	title="Resume where you left off"
	{message}
	actions={[
		{
			label: 'Resume',
			style: 'primary',
			onclick: onResume
		},
		{
			label: 'Start again',
			style: 'secondary',
			onclick: onStartAgain
		}
	]}
/>
