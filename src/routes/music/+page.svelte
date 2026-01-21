<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { getMusicTracks, getMusicTracksCount, playSound, stopSound, pauseSound, resumeSound, seekSound, setVolume, getPlaybackStatus, setWindowForMusicPlayer, setWindowForSoundboard } from '$lib/api';
	import type { MusicTrack } from '$lib/types';
	import { WinampPlayer, PlaylistPanel } from '$lib/components/music';

	let musicTracks = $state<MusicTrack[]>([]);
	let trackCount = $state(0);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Player state
	let currentTrack = $state<MusicTrack | null>(null);
	let isPlaying = $state(false);
	let isPaused = $state(false);
	let positionSecs = $state(0);
	let durationSecs = $state(0);
	let volume = $state(0.8);
	let sampleRate = $state(0);
	let bitrateKbps = $state(0);

	let statusPollInterval: ReturnType<typeof setInterval> | null = null;

	async function goToSoundboard() {
		await setWindowForSoundboard();
		goto('/');
	}

	async function loadTracks() {
		try {
			[musicTracks, trackCount] = await Promise.all([getMusicTracks(), getMusicTracksCount()]);
		} catch (e) {
			error = String(e);
			console.error('Failed to load music tracks:', e);
		} finally {
			loading = false;
		}
	}

	async function pollPlaybackStatus() {
		try {
			const status = await getPlaybackStatus();
			isPlaying = status.isPlaying;
			isPaused = status.isPaused;
			positionSecs = status.positionSecs;
			// Only update duration if backend reports a valid value (Vorbis doesn't report duration)
			if (status.durationSecs > 0) {
				durationSecs = status.durationSecs;
			}
			volume = status.volume;
			sampleRate = status.sampleRate;
			bitrateKbps = status.bitrateKbps;
		} catch (e) {
			console.error('Failed to get playback status:', e);
		}
	}

	function startPolling() {
		if (statusPollInterval) return;
		statusPollInterval = setInterval(pollPlaybackStatus, 250);
	}

	function stopPolling() {
		if (statusPollInterval) {
			clearInterval(statusPollInterval);
			statusPollInterval = null;
		}
	}

	async function handleTrackSelect(track: MusicTrack) {
		currentTrack = track;
		durationSecs = track.durationSecs;
		try {
			await playSound(track.id, track.filePath);
			isPlaying = true;
			isPaused = false;
			startPolling();
		} catch (e) {
			console.error('Failed to play track:', e);
		}
	}

	async function handlePlay() {
		if (currentTrack) {
			if (isPaused) {
				await resumeSound();
			} else {
				await playSound(currentTrack.id, currentTrack.filePath);
			}
			isPlaying = true;
			isPaused = false;
			startPolling();
		} else if (musicTracks.length > 0) {
			await handleTrackSelect(musicTracks[0]);
		}
	}

	async function handlePause() {
		await pauseSound();
		isPaused = true;
	}

	async function handleStop() {
		await stopSound();
		isPlaying = false;
		isPaused = false;
		positionSecs = 0;
		stopPolling();
	}

	function getCurrentTrackIndex(): number {
		if (!currentTrack) return -1;
		return musicTracks.findIndex(t => t.id === currentTrack?.id);
	}

	async function handlePrev() {
		const index = getCurrentTrackIndex();
		if (index > 0) {
			await handleTrackSelect(musicTracks[index - 1]);
		} else if (musicTracks.length > 0) {
			await handleTrackSelect(musicTracks[musicTracks.length - 1]);
		}
	}

	async function handleNext() {
		const index = getCurrentTrackIndex();
		if (index < musicTracks.length - 1) {
			await handleTrackSelect(musicTracks[index + 1]);
		} else if (musicTracks.length > 0) {
			await handleTrackSelect(musicTracks[0]);
		}
	}

	async function handleSeek(position: number) {
		await seekSound(position);
		positionSecs = position;
	}

	async function handleVolumeChange(newVolume: number) {
		await setVolume(newVolume);
		volume = newVolume;
	}

	onMount(async () => {
		// Resize window for compact music player
		await setWindowForMusicPlayer();
		await loadTracks();
		// Initial status check
		await pollPlaybackStatus();
		if (isPlaying) {
			startPolling();
		}
	});

	onDestroy(() => {
		stopPolling();
	});
</script>

<div class="music-page">
	{#if loading}
		<div class="status-message">Loading music library...</div>
	{:else if error}
		<div class="status-message error">{error}</div>
	{:else if trackCount === 0}
		<div class="no-music">
			<h2>No Music Extracted</h2>
			<p>Music tracks haven't been extracted yet.</p>
			<p class="hint">
				Go to the Soundboard and click "Rebuild" with the "Include game music" option checked.
			</p>
			<button class="back-button" onclick={goToSoundboard}>Back to Soundboard</button>
		</div>
	{:else}
		<div class="player-container">
			<WinampPlayer
				title={currentTrack?.title ?? 'Select a track to play'}
				{isPlaying}
				{isPaused}
				{positionSecs}
				{durationSecs}
				{volume}
				{sampleRate}
				{bitrateKbps}
				onPlay={handlePlay}
				onPause={handlePause}
				onStop={handleStop}
				onPrev={handlePrev}
				onNext={handleNext}
				onSeek={handleSeek}
				onVolumeChange={handleVolumeChange}
				onBack={goToSoundboard}
			/>
			<PlaylistPanel
				tracks={musicTracks}
				currentTrackId={currentTrack?.id ?? null}
				onTrackSelect={handleTrackSelect}
			/>
		</div>
	{/if}
</div>

<style>
	.music-page {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: #1a1a1a;
		align-items: center;
		justify-content: flex-start;
	}

	.status-message {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: #888;
		font-family: 'Courier New', monospace;
	}

	.status-message.error {
		color: #cc4444;
	}

	.no-music {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		text-align: center;
		padding: 2rem;
		font-family: 'Courier New', monospace;
	}

	.no-music h2 {
		color: #00cc00;
		margin-bottom: 0.5rem;
		font-size: 1rem;
	}

	.no-music p {
		color: #888;
		margin-bottom: 0.5rem;
		font-size: 0.8rem;
	}

	.no-music .hint {
		font-size: 0.75rem;
		margin-top: 1rem;
	}

	.back-button {
		margin-top: 1rem;
		padding: 0.5rem 1rem;
		background: #2a3a2a;
		border: 1px solid #3a5a3a;
		color: #00cc00;
		font-family: 'Courier New', monospace;
		font-size: 0.75rem;
		cursor: pointer;
		transition: background 0.2s;
	}

	.back-button:hover {
		background: #3a4a3a;
	}

	.player-container {
		display: flex;
		flex-direction: column;
		align-items: center;
	}
</style>
