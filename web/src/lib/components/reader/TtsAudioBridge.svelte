<script lang="ts">
	import type { VoicePersonaResponse } from '$lib/api/generated/types.gen';
	import TtsControlStrip from './TtsControlStrip.svelte';

	interface Props {
		playing: boolean;
		loading: boolean;
		currentTime: number;
		duration: number;
		speed: number;
		personas: VoicePersonaResponse[];
		selectedPersonaId: string | null;
		onPlay: () => void;
		onPause: () => void;
		onStop: () => void;
		onSkipBack: () => void;
		onSkipForward: () => void;
		onSeek: (time: number) => void;
		onSpeedChange: (speed: number) => void;
		onPersonaChange: (personaId: string) => void;
	}

	let {
		playing,
		loading,
		currentTime,
		duration,
		speed,
		personas,
		selectedPersonaId,
		onPlay,
		onPause,
		onStop,
		onSkipBack,
		onSkipForward,
		onSeek,
		onSpeedChange,
		onPersonaChange
	}: Props = $props();
</script>

<TtsControlStrip
	{playing}
	{loading}
	{currentTime}
	{duration}
	{speed}
	{personas}
	{selectedPersonaId}
	{onPlay}
	{onPause}
	{onStop}
	{onSkipBack}
	{onSkipForward}
	{onSeek}
	{onSpeedChange}
	{onPersonaChange}
/>
