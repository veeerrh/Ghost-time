<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface TimelineEntry {
    id: number;
    timestamp: number;
    app_name: string;
    window_title: string;
    duration_ms: number;
    is_idle: boolean;
    is_approved: boolean;
    matter_id: number | null;
    matter_code: string | null;
    client_name: string | null;
  }

  let selectedDate = $state(new Date().toISOString().split('T')[0]);
  let entries: TimelineEntry[] = $state([]);
  let loading = $state(false);
  let error = $state('');

  const totalMs = $derived(entries.filter(e => !e.is_idle).reduce((sum, e) => sum + e.duration_ms, 0));
  const approvedMs = $derived(entries.filter(e => e.is_approved).reduce((sum, e) => sum + e.duration_ms, 0));
  const unclassifiedCount = $derived(entries.filter(e => !e.matter_id && !e.is_idle).length);

  function formatDuration(ms: number): string {
    const totalMin = Math.floor(ms / 60000);
    const hrs = Math.floor(totalMin / 60);
    const min = totalMin % 60;
    return hrs > 0 ? `${hrs}h ${min}m` : `${min}m`;
  }

  function formatTime(ms: number): string {
    return new Date(ms).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  async function loadEntries() {
    loading = true;
    error = '';
    try {
      entries = await invoke('get_daily_summary', { date: selectedDate });
    } catch (e: any) {
      error = e.toString();
      entries = [];
    } finally {
      loading = false;
    }
  }

  async function approveEntry(id: number) {
    try {
      await invoke('approve_entry', { entryId: id });
      // Update local state
      entries = entries.map(e => e.id === id ? { ...e, is_approved: true } : e);
    } catch (e: any) {
      error = e.toString();
    }
  }

  $effect(() => {
    selectedDate; // reactive dependency
    loadEntries();
  });
</script>

<div class="page-header">
  <h2>📊 Daily Timeline</h2>
  <div style="display: flex; gap: 12px; align-items: center;">
    <input type="date" bind:value={selectedDate} />
    <button class="btn" onclick={loadEntries}>↻ Refresh</button>
  </div>
</div>

<div class="page-body">
  <!-- Stats Row -->
  <div class="stats-row">
    <div class="stat-card">
      <div class="label">Total Active</div>
      <div class="value accent">{formatDuration(totalMs)}</div>
    </div>
    <div class="stat-card">
      <div class="label">Approved</div>
      <div class="value success">{formatDuration(approvedMs)}</div>
    </div>
    <div class="stat-card">
      <div class="label">Entries</div>
      <div class="value">{entries.length}</div>
    </div>
    <div class="stat-card">
      <div class="label">Unclassified</div>
      <div class="value warning">{unclassifiedCount}</div>
    </div>
  </div>

  {#if error}
    <div class="card" style="border-color: var(--danger); color: var(--danger); margin-bottom: 16px;">
      ⚠️ {error}
    </div>
  {/if}

  {#if loading}
    <div class="empty-state">
      <div class="icon">⏳</div>
      <p>Loading entries...</p>
    </div>
  {:else if entries.length === 0}
    <div class="empty-state">
      <div class="icon">👻</div>
      <p>No activity recorded for this date. Ghost-Time logs entries as you work.</p>
    </div>
  {:else}
    <!-- Timeline Header -->
    <div class="timeline-entry" style="background: transparent; border: none; font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.8px;">
      <span>Time</span>
      <span>Window</span>
      <span>Duration</span>
      <span>Matter</span>
      <span>Action</span>
    </div>

    <div class="timeline-list">
      {#each entries as entry (entry.id)}
        <div
          class="timeline-entry"
          class:idle={entry.is_idle}
          class:unclassified={!entry.matter_id && !entry.is_idle}
          class:approved={entry.is_approved}
        >
          <span class="time">{formatTime(entry.timestamp)}</span>
          <div>
            <div class="title">{entry.window_title}</div>
            <div class="app">{entry.app_name}</div>
          </div>
          <span class="duration">
            {entry.is_idle ? '💤' : ''} {formatDuration(entry.duration_ms)}
          </span>
          <span class="matter-tag" class:none={!entry.matter_code}>
            {entry.matter_code || 'Untagged'}
          </span>
          <span>
            {#if entry.is_approved}
              <span class="badge badge-active">✓ Approved</span>
            {:else if !entry.is_idle}
              <button class="btn btn-sm btn-success" onclick={() => approveEntry(entry.id)}>
                Approve
              </button>
            {:else}
              <span class="badge badge-idle">Idle</span>
            {/if}
          </span>
        </div>
      {/each}
    </div>
  {/if}
</div>
