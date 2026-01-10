<script lang="ts">
	import type { Category } from '$lib/api';
	import { filterState } from '$lib/stores/sounds.svelte';

	let {
		categories,
		favoritesCount = 0
	}: { categories: Category[]; favoritesCount?: number } = $props();

	// Total count across all categories
	let totalCount = $derived(categories.reduce((sum, cat) => sum + cat.count, 0));

	function handleFavoritesClick() {
		filterState.showFavoritesOnly = true;
		filterState.category = null;
	}

	function handleCategoryClick(categoryId: string | null) {
		filterState.showFavoritesOnly = false;
		filterState.category = categoryId;
	}

	function isFavoritesSelected(): boolean {
		return filterState.showFavoritesOnly;
	}

	function isCategorySelected(categoryId: string | null): boolean {
		return !filterState.showFavoritesOnly && filterState.category === categoryId;
	}
</script>

<nav class="category-sidebar">
	<div class="sidebar-header">
		<h2>Categories</h2>
	</div>

	<ul class="category-list">
		<!-- Favorites option -->
		{#if favoritesCount > 0}
			<li>
				<button
					class="category-item"
					class:selected={isFavoritesSelected()}
					onclick={handleFavoritesClick}
				>
					<span class="category-name">Favorites</span>
					<span class="category-count">{favoritesCount}</span>
				</button>
			</li>
			<li class="divider"></li>
		{/if}

		<!-- All Sounds option -->
		<li>
			<button
				class="category-item"
				class:selected={isCategorySelected(null)}
				onclick={() => handleCategoryClick(null)}
			>
				<span class="category-name">All Sounds</span>
				<span class="category-count">{totalCount}</span>
			</button>
		</li>

		{#each categories as category (category.id)}
			<li>
				<button
					class="category-item"
					class:selected={isCategorySelected(category.id)}
					onclick={() => handleCategoryClick(category.id)}
				>
					<span class="category-name">{category.name}</span>
					<span class="category-count">{category.count}</span>
				</button>
			</li>
		{/each}
	</ul>
</nav>

<style>
	.category-sidebar {
		width: 220px;
		min-width: 220px;
		background: var(--color-bg-secondary);
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		overflow-y: auto;
	}

	.sidebar-header {
		padding: 1rem;
		border-bottom: 1px solid var(--color-border);
	}

	.sidebar-header h2 {
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-muted);
		margin: 0;
	}

	.category-list {
		list-style: none;
		padding: 0.5rem 0;
		margin: 0;
	}

	.divider {
		height: 1px;
		background: var(--color-border);
		margin: 0.5rem 1rem;
		opacity: 0.5;
	}

	.category-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
		padding: 0.625rem 1rem;
		background: transparent;
		border: none;
		text-align: left;
		color: var(--color-text);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.category-item:hover {
		background: var(--color-bg-tertiary);
	}

	.category-item.selected {
		background: var(--color-primary);
		color: white;
	}

	.category-name {
		font-size: 0.9rem;
	}

	.category-count {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		background: var(--color-bg);
		padding: 0.125rem 0.5rem;
		border-radius: var(--radius-sm);
	}

	.category-item.selected .category-count {
		background: rgba(255, 255, 255, 0.2);
		color: rgba(255, 255, 255, 0.9);
	}
</style>
