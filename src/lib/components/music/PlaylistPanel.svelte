<script lang="ts">
	import type { MusicTrack } from '$lib/types';

	let {
		tracks = [],
		currentTrackId = null,
		onTrackSelect
	}: {
		tracks?: MusicTrack[];
		currentTrackId?: string | null;
		onTrackSelect?: (track: MusicTrack) => void;
	} = $props();

	function formatDuration(secs: number): string {
		const mins = Math.floor(secs / 60);
		const s = Math.floor(secs % 60);
		return `${mins}:${s.toString().padStart(2, '0')}`;
	}
</script>

<div class="playlist-panel">
	<div class="playlist-header">
		<span class="playlist-title">Playlist</span>
		<span class="track-count">{tracks.length} tracks</span>
	</div>
	<div class="playlist-content">
		{#if tracks.length === 0}
			<div class="empty-message">No tracks loaded</div>
		{:else}
			<div class="track-list" role="list">
				{#each tracks as track, index}
					<button
						class="track-item"
						class:active={track.id === currentTrackId}
						onclick={() => onTrackSelect?.(track)}
						type="button"
					>
						<span class="track-number">{index + 1}.</span>
						<span class="track-title">{track.title}</span>
						<span class="track-duration">{formatDuration(track.durationSecs)}</span>
					</button>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.playlist-panel {
		--winamp-scale: 2;
		width: calc(275px * var(--winamp-scale));
		background: #232323;
		border: calc(2px * var(--winamp-scale)) solid #3a3a3a;
		border-top: none;
		font-family: 'Courier New', monospace;
		image-rendering: pixelated;
	}

	.playlist-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: calc(4px * var(--winamp-scale)) calc(8px * var(--winamp-scale));
		background: #1a1a1a;
		border-bottom: 1px solid #3a3a3a;
	}

	.playlist-title {
		font-size: calc(6px * var(--winamp-scale));
		color: #00cc00;
		text-transform: uppercase;
		letter-spacing: 1px;
	}

	.track-count {
		font-size: calc(5px * var(--winamp-scale));
		color: #888;
	}

	.playlist-content {
		max-height: calc(150px * var(--winamp-scale));
		overflow-y: auto;
	}

	.playlist-content::-webkit-scrollbar {
		width: calc(8px * var(--winamp-scale));
	}

	.playlist-content::-webkit-scrollbar-track {
		background: #1a1a1a;
	}

	.playlist-content::-webkit-scrollbar-thumb {
		background: #3a5a3a;
		border-radius: calc(2px * var(--winamp-scale));
	}

	.playlist-content::-webkit-scrollbar-thumb:hover {
		background: #4a7a4a;
	}

	.empty-message {
		padding: calc(16px * var(--winamp-scale));
		text-align: center;
		color: #666;
		font-size: calc(5px * var(--winamp-scale));
	}

	.track-list {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
	}

	.track-item {
		display: flex;
		align-items: center;
		padding: calc(3px * var(--winamp-scale)) calc(8px * var(--winamp-scale));
		gap: calc(4px * var(--winamp-scale));
		cursor: pointer;
		transition: background 0.1s;
		font-size: calc(5px * var(--winamp-scale));
		color: #00cc00;
		background: transparent;
		border: none;
		text-align: left;
		width: 100%;
		font-family: inherit;
	}

	.track-item:hover {
		background: #2a3a2a;
	}

	.track-item.active {
		background: #1a3a1a;
		color: #00ff00;
	}

	.track-number {
		color: #666;
		width: calc(16px * var(--winamp-scale));
		flex-shrink: 0;
	}

	.track-title {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.track-duration {
		color: #888;
		flex-shrink: 0;
		font-variant-numeric: tabular-nums;
	}
</style>
