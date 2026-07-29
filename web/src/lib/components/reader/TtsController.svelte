<script lang="ts">
	import { onDestroy } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import * as apiSdk from '$lib/api';
	import type {
		PlaybackStateResponse,
		SessionManifestResponse,
		VoicePersonaResponse
	} from '$lib/api/generated/types.gen';
	import TtsAudioBridge from './TtsAudioBridge.svelte';
	import TtsBanner from './TtsBanner.svelte';
	import TtsResumePrompt from './TtsResumePrompt.svelte';
	import {
		chunkContainsElement,
		countForwardReadyChunks,
		findChunkById,
		findNextReadyChunk,
		findReadyChunkForElement,
		getTtsUnavailableBanner,
		messageForTtsError,
		type TtsSessionChunk,
		type TtsStatus
	} from './tts-controller-model';
	import { clearTtsHighlight, collectTtsSpokenElements, setTtsHighlight } from './tts-dom';
	import {
		createTtsAudio,
		loadTtsChunkTimings,
		seekTtsAudioTo,
		waitForMetadata
	} from './tts-playback';
	import {
		createTtsResumeAnchor,
		selectActiveTtsElement,
		timestampForResumeAnchor,
		type TtsResumeAnchor,
		type TtsTimingEntry
	} from './tts-timing';

	interface Props {
		documentId: string;
		articleBodyEl: HTMLDivElement | undefined;
	}

	type PlayChunkOptions = {
		startElementIndex?: number;
		startTimestamp?: number;
		resumeAnchor?: TtsResumeAnchor;
	};

	let { documentId, articleBodyEl }: Props = $props();

	let status = $state<TtsStatus>('idle');
	let unavailableMessage = $state('');
	let session = $state<SessionManifestResponse | null>(null);
	let activeElementIndex = $state(0);
	let speed = $state(1);
	let personas = $state<VoicePersonaResponse[]>([]);
	let selectedPersonaId = $state<string | null>(null);
	let resumeState = $state<PlaybackStateResponse | null>(null);
	let showResumePrompt = $state(false);
	let currentChunkId = $state<string | null>(null);
	let currentTime = $state(0);
	let duration = $state(0);

	let audioEl: HTMLAudioElement | null = null;
	let domElements: HTMLElement[] = [];
	const timingCache = new SvelteMap<number, TtsTimingEntry>();
	let currentHighlightedEl: HTMLElement | null = null;
	let currentSpeakingPillEl: HTMLElement | null = null;
	let persistDebounce: ReturnType<typeof setTimeout> | null = null;
	let initialized = false;
	let lookaheadRefreshingForChunkId: string | null = null;
	let stopOnNextEnd = false;
	let playbackRequestId = 0;

	const bannerForStatus = $derived(
		status === 'unavailable' ? getTtsUnavailableBanner(unavailableMessage) : null
	);

	$effect(() => {
		if (!articleBodyEl) return;
		domElements = collectTtsSpokenElements(articleBodyEl);
	});

	$effect(() => {
		if (initialized) return;
		initialized = true;
		void init();
	});

	async function init() {
		try {
			const { data } = await apiSdk.listPersonas();
			if (data) {
				personas = data.personas.filter((persona) => persona.status === 'active');
				if (personas.length > 0 && personas[0]) {
					selectedPersonaId = personas[0].id;
				}
			}
		} catch {
			// Managed TTS may be disabled; server defaults still work.
		}

		try {
			const { data } = await apiSdk.getDocumentPlaybackState({
				path: { document_id: documentId },
				query: { kind: 'tts' }
			});
			if (data && data.element_index != null && data.element_index > 0) {
				resumeState = data;
				showResumePrompt = true;
				if (data.tts_voice_persona_id) {
					selectedPersonaId = data.tts_voice_persona_id;
				}
				if (data.playback_speed) {
					speed = data.playback_speed;
				}
			}
		} catch {
			// 404 means no saved playback state; start fresh.
		}

		if (!showResumePrompt) {
			await startPlayback(0);
		}
	}

	function isCurrentPlaybackRequest(requestId: number): boolean {
		return requestId === playbackRequestId;
	}

	async function startPlayback(
		startElementIndex: number,
		requestId = ++playbackRequestId,
		resumeAnchor?: TtsResumeAnchor
	) {
		status = 'loading';
		showResumePrompt = false;
		clearHighlight();
		timingCache.clear();
		currentChunkId = null;

		try {
			const { data: manifest, error } = await apiSdk.startDocumentTtsSession({
				path: { document_id: documentId },
				body: {
					voice_persona_id: selectedPersonaId,
					generation_scope: 'section',
					start_element_index: startElementIndex
				}
			});
			if (!isCurrentPlaybackRequest(requestId)) return;

			if (error || !manifest) {
				status = 'unavailable';
				unavailableMessage = messageForTtsError(error);
				return;
			}

			session = manifest;

			const readyChunk =
				findReadyChunkForElement(manifest, manifest.start.element_index) ??
				findChunkById(manifest, manifest.start.chunk_id) ??
				manifest.chunks.find((chunk) => chunk.state === 'ready' && chunk.audio_url);
			if (!readyChunk?.audio_url) {
				status = 'unavailable';
				unavailableMessage = 'Audio not yet available. Please try again shortly.';
				return;
			}

			await playChunk(
				manifest,
				readyChunk,
				{
					startElementIndex: manifest.start.element_index,
					startTimestamp: manifest.start.start_timestamp,
					resumeAnchor
				},
				requestId
			);
		} catch (error: unknown) {
			if (!isCurrentPlaybackRequest(requestId)) return;
			status = 'unavailable';
			unavailableMessage = messageForTtsError(error);
		}
	}

	async function refreshLookaheadWindow(
		chunk: TtsSessionChunk,
		requireCurrentChunk = false
	): Promise<SessionManifestResponse | null> {
		try {
			const { data: manifest, error } = await apiSdk.startDocumentTtsSession({
				path: { document_id: documentId },
				body: {
					voice_persona_id: selectedPersonaId,
					generation_scope: 'section',
					start_element_index: chunk.start_element_index
				}
			});
			if (error || !manifest) return null;
			if (requireCurrentChunk && currentChunkId !== chunk.chunk_id) return null;
			session = manifest;
			return manifest;
		} catch {
			return null;
		}
	}

	async function ensureLookahead(manifest: SessionManifestResponse, chunk: TtsSessionChunk) {
		if (countForwardReadyChunks(manifest, chunk) >= 2) return;
		if (lookaheadRefreshingForChunkId === chunk.chunk_id) return;
		lookaheadRefreshingForChunkId = chunk.chunk_id;
		try {
			await refreshLookaheadWindow(chunk, true);
		} finally {
			if (lookaheadRefreshingForChunkId === chunk.chunk_id) {
				lookaheadRefreshingForChunkId = null;
			}
		}
	}

	async function playChunk(
		manifest: SessionManifestResponse,
		chunk: TtsSessionChunk,
		options: PlayChunkOptions = {},
		requestId = playbackRequestId
	) {
		if (!isCurrentPlaybackRequest(requestId)) return;
		if (!chunk.audio_url) {
			status = 'unavailable';
			unavailableMessage = 'Audio not yet available. Please try again shortly.';
			return;
		}

		status = 'loading';
		session = manifest;
		stopOnNextEnd = false;
		await loadTtsChunkTimings(documentId, manifest, chunk, timingCache);
		if (!isCurrentPlaybackRequest(requestId)) return;

		teardownAudio();
		audioEl = createTtsAudio(chunk.audio_url, apiSdk.getApiBaseUrl(), speed);
		const playbackAudio = audioEl;
		currentTime = 0;
		duration = 0;
		audioEl.addEventListener('timeupdate', handleTimeUpdate);
		audioEl.addEventListener('durationchange', handleDurationChange);
		audioEl.addEventListener('loadedmetadata', handleDurationChange);
		audioEl.addEventListener('ended', handleAudioEnded);
		audioEl.addEventListener('error', handleAudioError);

		currentChunkId = chunk.chunk_id;
		const canUseResumeAnchor =
			options.resumeAnchor != null &&
			chunkContainsElement(chunk, options.resumeAnchor.elementIndex);
		const targetTimestamp = canUseResumeAnchor
			? timestampForResumeAnchor(timingCache, options.resumeAnchor!, chunk)
			: (options.startTimestamp ??
				timingCache.get(options.startElementIndex ?? chunk.start_element_index)?.start ??
				0);
		const targetElementIndex = canUseResumeAnchor
			? selectActiveTtsElement(
					chunk,
					timingCache,
					targetTimestamp,
					chunk.duration_seconds ?? duration
				)
			: (options.startElementIndex ?? chunk.start_element_index);
		if (targetTimestamp > 0) {
			if (playbackAudio.readyState < 1) {
				await waitForMetadata(playbackAudio);
			}
			if (!isCurrentPlaybackRequest(requestId)) return;
			if (audioEl === playbackAudio) {
				currentTime = await seekTtsAudioTo(playbackAudio, targetTimestamp);
			}
		}

		if (audioEl !== playbackAudio || !isCurrentPlaybackRequest(requestId)) return;
		await playbackAudio.play();
		if (!isCurrentPlaybackRequest(requestId)) {
			if (audioEl === playbackAudio) {
				teardownAudio();
			}
			return;
		}
		status = 'playing';
		activeElementIndex = targetElementIndex;
		highlightElement(activeElementIndex);
		void ensureLookahead(manifest, chunk);
	}

	function handleTimeUpdate() {
		if (!audioEl || !session) return;
		currentTime = audioEl.currentTime;
		const currentChunk = findChunkById(session, currentChunkId);
		if (!currentChunk) return;

		const newIndex = selectActiveTtsElement(currentChunk, timingCache, currentTime, duration);

		if (newIndex !== activeElementIndex) {
			activeElementIndex = newIndex;
			highlightElement(newIndex);
			schedulePersist();
		}
	}

	function handleDurationChange() {
		if (!audioEl) return;
		duration = Number.isFinite(audioEl.duration) ? audioEl.duration : 0;
	}

	function handleAudioEnded() {
		if (stopOnNextEnd) {
			stopOnNextEnd = false;
			finishPlayback();
			return;
		}
		void advanceToNextChunk();
	}

	async function advanceToNextChunk() {
		if (!session) {
			finishPlayback();
			return;
		}

		let activeSession = session;
		let nextChunk = findNextReadyChunk(activeSession, currentChunkId);
		if (!nextChunk) {
			const currentChunk = findChunkById(activeSession, currentChunkId);
			if (currentChunk) {
				const refreshed = await refreshLookaheadWindow(currentChunk);
				if (refreshed) {
					activeSession = refreshed;
					nextChunk = findNextReadyChunk(activeSession, currentChunkId);
				}
			}
		}

		if (!nextChunk) {
			finishPlayback();
			return;
		}

		await playChunk(activeSession, nextChunk, {
			startElementIndex: nextChunk.start_element_index,
			startTimestamp: 0
		});
	}

	function finishPlayback() {
		status = 'paused';
		clearHighlight();
		void persistPosition();
		teardownAudio();
		session = null;
		currentChunkId = null;
		lookaheadRefreshingForChunkId = null;
		timingCache.clear();
		activeElementIndex = 0;
		currentTime = 0;
		duration = 0;
		stopOnNextEnd = false;
	}

	function handleAudioError() {
		status = 'unavailable';
		unavailableMessage = 'Audio playback failed. Please try again.';
		clearHighlight();
	}

	function clearHighlight() {
		clearTtsHighlight(currentHighlightedEl, currentSpeakingPillEl);
		currentHighlightedEl = null;
		currentSpeakingPillEl = null;
	}

	function highlightElement(elementIndex: number, instant = false) {
		clearHighlight();
		const next = setTtsHighlight(domElements, elementIndex, instant);
		currentHighlightedEl = next.highlightedEl;
		currentSpeakingPillEl = next.speakingPillEl;
	}

	function schedulePersist() {
		if (persistDebounce) clearTimeout(persistDebounce);
		persistDebounce = setTimeout(() => {
			void persistPosition();
		}, 3000);
	}

	async function persistPosition() {
		if (!session || !audioEl) return;
		try {
			await apiSdk.upsertDocumentPlaybackState({
				path: { document_id: documentId },
				body: {
					playback_kind: 'tts',
					position_seconds: audioEl.currentTime,
					playback_speed: speed,
					element_index: activeElementIndex,
					tts_chunk_id: currentChunkId ?? session.start.chunk_id,
					tts_voice_persona_id: selectedPersonaId,
					is_playing: status === 'playing'
				}
			});
		} catch {
			// Persistence failure is non-fatal; playback continues locally.
		}
	}

	async function handlePlay() {
		if (status === 'paused' && audioEl) {
			try {
				await audioEl.play();
				status = 'playing';
			} catch {
				status = 'unavailable';
				unavailableMessage = 'Playback could not resume. Please try again.';
			}
		} else {
			await startPlayback(activeElementIndex);
		}
	}

	async function handlePause() {
		audioEl?.pause();
		status = 'paused';
		await persistPosition();
	}

	async function handleStop() {
		audioEl?.pause();
		clearHighlight();
		await persistPosition();
		teardownAudio();
		session = null;
		status = 'idle';
		activeElementIndex = 0;
		currentChunkId = null;
		lookaheadRefreshingForChunkId = null;
		currentTime = 0;
		duration = 0;
		timingCache.clear();
		stopOnNextEnd = false;
	}

	function handleSkipBack() {
		if (!audioEl) return;
		audioEl.currentTime = Math.max(0, audioEl.currentTime - 15);
		currentTime = audioEl.currentTime;
		schedulePersist();
	}

	function handleSkipForward() {
		if (!audioEl) return;
		const upper = duration > 0 ? duration : audioEl.duration;
		const target = audioEl.currentTime + 15;
		audioEl.currentTime = Number.isFinite(upper) ? Math.min(upper, target) : target;
		currentTime = audioEl.currentTime;
		schedulePersist();
	}

	async function handleSeek(time: number) {
		if (!audioEl || !session) return;
		if (audioEl.readyState < 1) {
			await waitForMetadata(audioEl);
		}
		if (!audioEl) return;
		const upper = duration > 0 ? duration : audioEl.duration;
		const clamped = Math.max(0, Number.isFinite(upper) ? Math.min(upper, time) : time);
		audioEl.currentTime = clamped;
		currentTime = clamped;

		const chunk = findChunkById(session, currentChunkId);
		if (chunk) {
			const newIndex = selectActiveTtsElement(chunk, timingCache, clamped, Number(upper));
			if (newIndex !== activeElementIndex) {
				activeElementIndex = newIndex;
				highlightElement(newIndex, true);
			}
		}

		stopOnNextEnd = Number.isFinite(upper) && upper > 0 && clamped >= upper - 1.5;
		schedulePersist();
	}

	function handleSpeedChange(newSpeed: number) {
		speed = newSpeed;
		if (audioEl) {
			audioEl.playbackRate = newSpeed;
		}
		schedulePersist();
	}

	async function handlePersonaChange(personaId: string) {
		if (personaId === selectedPersonaId) return;
		const requestId = ++playbackRequestId;
		const currentChunk = session ? findChunkById(session, currentChunkId) : null;
		const playbackTime = audioEl?.currentTime ?? currentTime;
		const observedDuration =
			audioEl && Number.isFinite(audioEl.duration) && audioEl.duration > 0
				? audioEl.duration
				: duration;
		const resumeAnchor = currentChunk
			? createTtsResumeAnchor(currentChunk, timingCache, playbackTime, observedDuration)
			: { elementIndex: activeElementIndex, progressInElement: 0, progressInChunk: null };
		selectedPersonaId = personaId;
		if (status === 'playing' || status === 'paused' || status === 'loading') {
			audioEl?.pause();
			teardownAudio();
			currentChunkId = null;
			lookaheadRefreshingForChunkId = null;
			currentTime = 0;
			duration = 0;
			timingCache.clear();
			stopOnNextEnd = false;
			status = 'loading';
			await startPlayback(resumeAnchor.elementIndex, requestId, resumeAnchor);
		}
	}

	function teardownAudio() {
		if (!audioEl) return;
		audioEl.removeEventListener('timeupdate', handleTimeUpdate);
		audioEl.removeEventListener('durationchange', handleDurationChange);
		audioEl.removeEventListener('loadedmetadata', handleDurationChange);
		audioEl.removeEventListener('ended', handleAudioEnded);
		audioEl.removeEventListener('error', handleAudioError);
		audioEl.pause();
		audioEl.src = '';
		audioEl = null;
	}

	onDestroy(() => {
		if (persistDebounce) clearTimeout(persistDebounce);
		void persistPosition();
		teardownAudio();
		clearHighlight();
	});
</script>

{#if status === 'unavailable' && bannerForStatus}
	<TtsBanner
		variant={bannerForStatus.variant}
		title={bannerForStatus.title}
		message={bannerForStatus.message}
	/>
{:else if showResumePrompt && resumeState}
	<TtsResumePrompt
		positionSeconds={resumeState.position_seconds ?? 0}
		onResume={() => {
			void startPlayback(resumeState?.element_index ?? 0);
		}}
		onStartAgain={() => {
			void startPlayback(0);
		}}
	/>
{:else}
	<TtsAudioBridge
		playing={status === 'playing'}
		loading={status === 'loading'}
		{currentTime}
		{duration}
		{speed}
		{personas}
		{selectedPersonaId}
		onPlay={handlePlay}
		onPause={handlePause}
		onStop={handleStop}
		onSkipBack={handleSkipBack}
		onSkipForward={handleSkipForward}
		onSeek={handleSeek}
		onSpeedChange={handleSpeedChange}
		onPersonaChange={handlePersonaChange}
	/>
{/if}
