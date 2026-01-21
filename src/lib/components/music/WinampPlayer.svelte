<script lang="ts">
	import LCDDisplay from './LCDDisplay.svelte';
	import TextTicker from './TextTicker.svelte';
	import TransportControls from './TransportControls.svelte';
	import SeekBar from './SeekBar.svelte';
	import VolumeSlider from './VolumeSlider.svelte';
	import PlayIndicator from './PlayIndicator.svelte';
	import Visualizer from './Visualizer.svelte';
	import SmallDigits from './SmallDigits.svelte';

	let {
		title = 'No track loaded',
		isPlaying = false,
		isPaused = false,
		positionSecs = 0,
		durationSecs = 0,
		volume = 0.8,
		sampleRate = 0,
		bitrateKbps = 0,
		onPlay,
		onPause,
		onStop,
		onPrev,
		onNext,
		onSeek,
		onVolumeChange,
		onBack
	}: {
		title?: string;
		isPlaying?: boolean;
		isPaused?: boolean;
		positionSecs?: number;
		durationSecs?: number;
		volume?: number;
		sampleRate?: number;
		bitrateKbps?: number;
		onPlay?: () => void;
		onPause?: () => void;
		onStop?: () => void;
		onPrev?: () => void;
		onNext?: () => void;
		onSeek?: (position: number) => void;
		onVolumeChange?: (volume: number) => void;
		onBack?: () => void;
	} = $props();

	let minutes = $derived(Math.floor(positionSecs / 60));
	let seconds = $derived(Math.floor(positionSecs % 60));

	// Format kHz (44100 -> 44)
	let kHz = $derived(sampleRate > 0 ? Math.round(sampleRate / 1000) : 0);
</script>

<div class="winamp-player">
	<div class="winamp-main">
		<!-- Title bar area -->
		<div class="titlebar">
			{#if onBack}
				<button class="back-btn" onclick={onBack} title="Back to Soundboard">
					<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
						<path d="m12 19-7-7 7-7M19 12H5" />
					</svg>
				</button>
			{/if}
			<span class="winamp-title">Kithara</span>
		</div>

		<!-- Play state indicator -->
		<div class="play-indicator-area">
			<PlayIndicator {isPlaying} {isPaused} />
		</div>

		<!-- Time display -->
		<div class="time-display">
			<LCDDisplay {minutes} {seconds} />
		</div>

		<!-- Visualizer - in the spectrum analyzer area -->
		<div class="visualizer-area">
			<Visualizer {isPlaying} {isPaused} />
		</div>

		<!-- Bitrate display (kbps) -->
		<div class="kbps-area">
			<SmallDigits value={bitrateKbps} padTo={3} />
		</div>

		<!-- Sample rate display (kHz) -->
		<div class="khz-area">
			<SmallDigits value={kHz} padTo={2} />
		</div>

		<!-- Track info ticker -->
		<div class="ticker-area">
			<TextTicker text={title} />
		</div>


		<!-- Seek bar -->
		<div class="seek-area">
			<SeekBar position={positionSecs} duration={durationSecs} {onSeek} />
		</div>

		<!-- Volume slider -->
		<div class="volume-area">
			<VolumeSlider {volume} {onVolumeChange} />
		</div>

		<!-- Transport controls -->
		<div class="transport-area">
			<TransportControls
				{isPlaying}
				{isPaused}
				{onPlay}
				{onPause}
				{onStop}
				{onPrev}
				{onNext}
			/>
		</div>
	</div>
</div>

<style>
	.winamp-player {
		--winamp-scale: 2;
		--winamp-width: calc(275px * var(--winamp-scale));
		--winamp-height: calc(116px * var(--winamp-scale));
		width: var(--winamp-width);
		height: var(--winamp-height);
		image-rendering: pixelated;
		font-family: monospace;
		user-select: none;
	}

	.winamp-main {
		width: 100%;
		height: 100%;
		background-image: url('/skins/winamp/MAIN.BMP');
		background-size: var(--winamp-width) auto;
		background-repeat: no-repeat;
		position: relative;
		border-radius: 4px;
		overflow: hidden;
	}

	.titlebar {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: calc(14px * var(--winamp-scale));
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.back-btn {
		position: absolute;
		left: calc(4px * var(--winamp-scale));
		background: none;
		border: none;
		padding: calc(2px * var(--winamp-scale));
		cursor: pointer;
		color: #00aa00;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 2px;
		transition: color 0.2s, background 0.2s;
	}

	.back-btn:hover {
		color: #00ff00;
		background: rgba(0, 255, 0, 0.1);
	}

	.back-btn svg {
		width: calc(8px * var(--winamp-scale));
		height: calc(8px * var(--winamp-scale));
	}

	.winamp-title {
		font-size: calc(8px * var(--winamp-scale));
		color: #00ff00;
		text-shadow: 0 0 2px #00ff00;
		letter-spacing: 1px;
	}

	/* Play indicator - small icon left of time */
	.play-indicator-area {
		position: absolute;
		top: calc(28px * var(--winamp-scale));
		left: calc(26px * var(--winamp-scale));
		width: calc(9px * var(--winamp-scale));
		height: calc(9px * var(--winamp-scale));
	}

	/* Time display - main LCD numbers */
	.time-display {
		position: absolute;
		top: calc(26px * var(--winamp-scale));
		left: calc(48px * var(--winamp-scale));
	}

	/* Visualizer - spectrum area to the right */
	.visualizer-area {
		position: absolute;
		top: calc(43px * var(--winamp-scale));
		left: calc(24px * var(--winamp-scale));
		width: calc(76px * var(--winamp-scale));
		height: calc(16px * var(--winamp-scale));
		overflow: hidden;
	}

	/* Bitrate display (kbps) - below time, left of ticker */
	.kbps-area {
		position: absolute;
		top: calc(43px * var(--winamp-scale));
		left: calc(111px * var(--winamp-scale));
	}

	/* Sample rate display (kHz) - right of kbps */
	.khz-area {
		position: absolute;
		top: calc(43px * var(--winamp-scale));
		left: calc(156px * var(--winamp-scale));
	}

	/* Track info ticker - scrolling text */
	.ticker-area {
		position: absolute;
		top: calc(27px * var(--winamp-scale));
		left: calc(111px * var(--winamp-scale));
		width: calc(152px * var(--winamp-scale));
		height: calc(15px * var(--winamp-scale));
		overflow: hidden;
	}

	/* Seek/position bar */
	.seek-area {
		position: absolute;
		top: calc(72px * var(--winamp-scale));
		left: calc(16px * var(--winamp-scale));
		width: calc(248px * var(--winamp-scale));
	}

	/* Volume slider */
	.volume-area {
		position: absolute;
		top: calc(57px * var(--winamp-scale));
		left: calc(107px * var(--winamp-scale));
		width: calc(68px * var(--winamp-scale));
	}

	/* Transport controls - play/pause/stop etc */
	.transport-area {
		position: absolute;
		top: calc(88px * var(--winamp-scale));
		left: calc(16px * var(--winamp-scale));
	}
</style>
