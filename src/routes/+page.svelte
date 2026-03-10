<script lang="ts">
  import { currentPage } from '$lib/stores';
  import Timeline from '$lib/components/Timeline.svelte';
  import Matters from '$lib/components/Matters.svelte';
  import Export from '$lib/components/Export.svelte';
</script>

{#if $currentPage === 'timeline'}
  <Timeline />
{:else if $currentPage === 'matters'}
  <Matters />
{:else if $currentPage === 'export'}
  <Export />
{/if}
