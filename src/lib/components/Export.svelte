<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let startDate = $state(new Date().toISOString().split('T')[0]);
  let endDate = $state(new Date().toISOString().split('T')[0]);
  let exporting = $state(false);
  let exportResult = $state('');
  let error = $state('');

  async function doExport(format: string) {
    exporting = true;
    error = '';
    exportResult = '';
    try {
      const filePath: string = await invoke('export_timesheet', {
        dateRange: [startDate, endDate],
        format,
      });
      exportResult = filePath;
    } catch (e: any) {
      error = e.toString();
    } finally {
      exporting = false;
    }
  }
</script>

<div class="page-header">
  <h2>📤 Export Timesheet</h2>
</div>

<div class="page-body">
  <div class="card" style="max-width: 600px;">
    <h3 style="font-size: 16px; margin-bottom: 20px; color: var(--text-secondary);">
      Export approved time entries as a billing-ready document.
    </h3>

    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 24px;">
      <div class="form-group">
        <label>Start Date</label>
        <input type="date" bind:value={startDate} />
      </div>
      <div class="form-group">
        <label>End Date</label>
        <input type="date" bind:value={endDate} />
      </div>
    </div>

    <div style="display: flex; gap: 12px; margin-bottom: 24px;">
      <button
        class="btn btn-primary"
        onclick={() => doExport('csv')}
        disabled={exporting}
      >
        {exporting ? '⏳ Exporting...' : '📄 Export CSV'}
      </button>
      <button
        class="btn btn-primary"
        onclick={() => doExport('pdf')}
        disabled={exporting}
      >
        {exporting ? '⏳ Exporting...' : '📑 Export PDF'}
      </button>
    </div>

    {#if error}
      <div class="card" style="border-color: var(--danger); color: var(--danger);">
        ⚠️ {error}
      </div>
    {/if}

    {#if exportResult}
      <div class="card" style="border-color: var(--success); background: var(--success-bg);">
        <div style="color: var(--success); font-weight: 600; margin-bottom: 8px;">✅ Export Complete</div>
        <div style="font-size: 13px; color: var(--text-secondary); word-break: break-all;">
          Saved to: <code>{exportResult}</code>
        </div>
      </div>
    {/if}
  </div>

  <div class="card" style="max-width: 600px; margin-top: 16px;">
    <h3 style="font-size: 14px; color: var(--text-muted); margin-bottom: 12px;">ℹ️ Export Details</h3>
    <ul style="font-size: 13px; color: var(--text-secondary); list-style: disc; padding-left: 20px; line-height: 1.8;">
      <li>Only <strong>approved</strong> entries are included.</li>
      <li>CSV includes: Date, Time, App, Window Title, Duration, Matter, Client, Rate, and Amount.</li>
      <li>Amount is calculated as: <code>(duration_min / 60) × rate_per_hour</code>.</li>
    </ul>
  </div>
</div>
