<script lang="ts">
	import type { UnitType } from '$lib/api';
	import { filterState } from '$lib/stores/sounds.svelte';

	let { unitTypes }: { unitTypes: UnitType[] } = $props();

	function handleChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		filterState.unitType = target.value || null;
	}
</script>

<div class="unit-filter">
	<label for="unit-type-select" class="filter-label">Unit</label>
	<select
		id="unit-type-select"
		class="unit-select"
		value={filterState.unitType ?? ''}
		onchange={handleChange}
	>
		<option value="">All Units</option>
		{#each unitTypes as unit (unit.id)}
			<option value={unit.id}>{unit.name} ({unit.count})</option>
		{/each}
	</select>
</div>

<style>
	.unit-filter {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.filter-label {
		font-size: 0.8rem;
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.unit-select {
		padding: 0.5rem 2rem 0.5rem 0.75rem;
		background: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		color: var(--color-text);
		font-size: 0.85rem;
		cursor: pointer;
		outline: none;
		appearance: none;
		background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23a0a0a0' stroke-width='2'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
		background-repeat: no-repeat;
		background-position: right 0.5rem center;
		min-width: 150px;
	}

	.unit-select:focus {
		border-color: var(--color-primary);
	}

	.unit-select:hover {
		border-color: var(--color-text-muted);
	}
</style>
