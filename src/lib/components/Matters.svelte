<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface Matter {
    id: number | null;
    code: string;
    client_name: string;
    keywords: string[];
    rate_cents: number;
  }

  let matters: Matter[] = $state([]);
  let loading = $state(false);
  let error = $state('');

  // Form state
  let editingId: number | null = $state(null);
  let formCode = $state('');
  let formClient = $state('');
  let formKeywords = $state('');
  let formRate = $state('');

  function resetForm() {
    editingId = null;
    formCode = '';
    formClient = '';
    formKeywords = '';
    formRate = '';
  }

  function editMatter(m: Matter) {
    editingId = m.id;
    formCode = m.code;
    formClient = m.client_name;
    formKeywords = m.keywords.join(', ');
    formRate = (m.rate_cents / 100).toFixed(2);
  }

  async function saveMatter() {
    error = '';
    const keywords = formKeywords.split(',').map(k => k.trim()).filter(k => k.length > 0);
    const rateCents = Math.round(parseFloat(formRate || '0') * 100);
    
    try {
      const newId: number = await invoke('upsert_matter', {
        matter: {
          id: editingId,
          code: formCode,
          client_name: formClient,
          keywords,
          rate_cents: rateCents,
        }
      });
      
      if (editingId) {
        matters = matters.map(m => m.id === editingId ? {
          id: editingId, code: formCode, client_name: formClient, keywords, rate_cents: rateCents
        } : m);
      } else {
        matters = [...matters, {
          id: newId, code: formCode, client_name: formClient, keywords, rate_cents: rateCents
        }];
      }
      resetForm();
    } catch (e: any) {
      error = e.toString();
    }
  }
</script>

<div class="page-header">
  <h2>📁 Matter Manager</h2>
  <button class="btn btn-primary" onclick={resetForm}>
    + New Matter
  </button>
</div>

<div class="page-body">
  {#if error}
    <div class="card" style="border-color: var(--danger); color: var(--danger); margin-bottom: 16px;">
      ⚠️ {error}
    </div>
  {/if}

  <!-- Form -->
  <div class="card" style="margin-bottom: 24px;">
    <div style="display: grid; grid-template-columns: 1fr 1fr 2fr 1fr auto; gap: 12px; align-items: end;">
      <div class="form-group">
        <label>Matter Code</label>
        <input type="text" placeholder="SMITH-990" bind:value={formCode} />
      </div>
      <div class="form-group">
        <label>Client Name</label>
        <input type="text" placeholder="Smith & Associates" bind:value={formClient} />
      </div>
      <div class="form-group">
        <label>Keywords (comma-separated)</label>
        <input type="text" placeholder="Smith, Smith v Doe, Case 990" bind:value={formKeywords} />
      </div>
      <div class="form-group">
        <label>Rate ($/hr)</label>
        <input type="text" placeholder="250.00" bind:value={formRate} />
      </div>
      <div class="form-group">
        <label>&nbsp;</label>
        <button class="btn btn-primary" onclick={saveMatter}>
          {editingId ? 'Update' : 'Add'}
        </button>
      </div>
    </div>
  </div>

  <!-- Table -->
  {#if matters.length === 0}
    <div class="empty-state">
      <div class="icon">📁</div>
      <p>No matters configured yet. Add a client matter above to start classifying your time.</p>
    </div>
  {:else}
    <table class="data-table">
      <thead>
        <tr>
          <th>Code</th>
          <th>Client</th>
          <th>Keywords</th>
          <th>Rate</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each matters as matter (matter.id)}
          <tr>
            <td><strong>{matter.code}</strong></td>
            <td>{matter.client_name}</td>
            <td>
              {#each matter.keywords as kw}
                <span class="badge badge-active" style="margin-right: 4px;">{kw}</span>
              {/each}
            </td>
            <td>${(matter.rate_cents / 100).toFixed(2)}/hr</td>
            <td>
              <button class="btn btn-sm" onclick={() => editMatter(matter)}>✏️ Edit</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
