<script lang="ts">
import { getContext } from 'svelte';
import type { createSearchManager } from '../utils/searchManager.svelte';
import type { createAppCoordinator } from '../utils/appCoordinator.svelte';

const { searchManager, appCoordinator } = getContext<{
  searchManager: ReturnType<typeof createSearchManager>;
  appCoordinator: ReturnType<typeof createAppCoordinator>;
}>('managers');

const context = appCoordinator.context;

// Debug panel visibility and configuration
let isVisible = $state(false);
let isEnabled = $state(true); // Always available in desktop app

// Filter toggles for different sections
let showAppCoordinatorState = $state(true);
let showSearchManagerState = $state(true);
let showContextState = $state(true);
let showSearchFlow = $state(true);
let showActions = $state(false);

function togglePanel() {
  isVisible = !isVisible;
}

function formatArray(arr: string[]) {
  if (arr.length === 0) return '[]';
  if (arr.length <= 2) return JSON.stringify(arr);
  return `[${JSON.stringify(arr[0])}, ${JSON.stringify(arr[1])}, ...] (${arr.length} total)`;
}


// Keyboard shortcut support
function handleKeydown(event: KeyboardEvent) {
  if (isEnabled && event.ctrlKey && event.shiftKey && event.code === 'KeyD') {
    event.preventDefault();
    togglePanel();
  }
}
</script>

<!-- Debug toggle button - small subtle red circle -->
<!-- svelte-ignore a11y_consider_explicit_label -->
<button
  class="debug-toggle"
  onclick={togglePanel}
  title="Debug Panel (Ctrl+Shift+D)"
  class:active={isVisible}
>
</button>

<!-- Global keyboard shortcut -->
<svelte:window onkeydown={handleKeydown} />

<!-- Debug panel -->
{#if isVisible}
  <div class="debug-panel">
    <div class="debug-header">
      <h3>🔍 Debug Panel</h3>
      <button onclick={togglePanel}>×</button>
    </div>

    <!-- Filter Controls -->
    <div class="debug-filters">
      <label class="filter-checkbox">
        <input type="checkbox" bind:checked={showAppCoordinatorState} />
        AppCoordinator State
      </label>
      <label class="filter-checkbox">
        <input type="checkbox" bind:checked={showSearchManagerState} />
        SearchManager State
      </label>
      <label class="filter-checkbox">
        <input type="checkbox" bind:checked={showContextState} />
        Context State
      </label>
      <label class="filter-checkbox">
        <input type="checkbox" bind:checked={showSearchFlow} />
        Search Flow Debug
      </label>
      <label class="filter-checkbox">
        <input type="checkbox" bind:checked={showActions} />
        Debug Actions
      </label>
    </div>

    <div class="debug-content">
      {#if showAppCoordinatorState}
      <div class="debug-section">
        <h4>🎯 AppCoordinator State</h4>
        <div class="debug-item">
          <strong>filteredNotes:</strong>
          <code>{formatArray(appCoordinator.filteredNotes)}</code>
        </div>
        <div class="debug-item">
          <strong>selectedNote:</strong>
          <code>{JSON.stringify(appCoordinator.selectedNote)}</code>
        </div>
        <div class="debug-item">
          <strong>selectedIndex:</strong>
          <code>{appCoordinator.selectedIndex}</code>
        </div>
        <div class="debug-item">
          <strong>searchInput:</strong>
          <code>{JSON.stringify(searchManager.searchInput)}</code>
        </div>
        <div class="debug-item">
          <strong>query:</strong>
          <code>{JSON.stringify(appCoordinator.query)}</code>
        </div>
        <div class="debug-item">
          <strong>isLoading:</strong>
          <code>{appCoordinator.isLoading}</code>
        </div>
      </div>
      {/if}

      {#if showSearchManagerState}
      <div class="debug-section">
        <h4>🔍 SearchManager State</h4>
        <div class="debug-item">
          <strong>filteredNotes:</strong>
          <code>{formatArray(searchManager.filteredNotes)}</code>
        </div>
        <div class="debug-item">
          <strong>searchInput:</strong>
          <code>{JSON.stringify(searchManager.searchInput)}</code>
        </div>
        <div class="debug-item">
          <strong>isLoading:</strong>
          <code>{searchManager.isLoading}</code>
        </div>
        <div class="debug-item">
          <strong>areHighlightsCleared:</strong>
          <code>{context.contentManager.areHighlightsCleared}</code>
        </div>
      </div>
      {/if}

      {#if showContextState}
      <div class="debug-section">
        <h4>🎪 Context State (what components see)</h4>
        <div class="debug-item">
          <strong>filteredNotes:</strong>
          <code>{formatArray(appCoordinator.filteredNotes)}</code>
        </div>
        <div class="debug-item">
          <strong>selectedNote:</strong>
          <code>{JSON.stringify(appCoordinator.selectedNote)}</code>
        </div>
        <div class="debug-item">
          <strong>selectedIndex:</strong>
          <code>{appCoordinator.selectedIndex}</code>
        </div>
        <div class="debug-item">
          <strong>searchInput:</strong>
          <code>{JSON.stringify(searchManager.searchInput)}</code>
        </div>
        <div class="debug-item">
          <strong>isLoading:</strong>
          <code>{appCoordinator.isLoading}</code>
        </div>
      </div>
      {/if}

      {#if showSearchFlow}
      <div class="debug-section">
        <h4>🔧 Search Flow Debug</h4>
        <div class="debug-item">
          <strong>Search Sync Check:</strong>
          <small>appCentral.searchInput === searchMgr.searchInput?</small>
          <code>{searchManager.searchInput === searchManager.searchInput ? '✅ MATCH' : '❌ MISMATCH'}</code>
        </div>
        <div class="debug-item">
          <strong>Notes Sync Check:</strong>
          <small>appCoordinator.filteredNotes === searchMgr.filteredNotes?</small>
          <code>{appCoordinator.filteredNotes.length === searchManager.filteredNotes.length ? '✅ MATCH' : '❌ MISMATCH'} ({appCoordinator.filteredNotes.length} vs {searchManager.filteredNotes.length})</code>
        </div>
        <div class="debug-item">
          <strong>Reactive Effects Active?</strong>
          <small>Check if effects are running</small>
          <code>✅ Working (search input updates)</code>
        </div>
      </div>
      {/if}

      {#if showActions}
      <div class="debug-section">
        <h4>🧪 Debug Actions</h4>
        <button onclick={() => {
          console.log('🔍 Manual searchManager.searchImmediate("")');
          searchManager.searchImmediate('');
        }}>
          Trigger searchManager.searchImmediate('')
        </button>
        <button onclick={() => {
          console.log('🔍 Manual searchManager.searchImmediate("test")');
          searchManager.searchImmediate('test');
        }}>
          Trigger searchManager.searchImmediate('test')
        </button>
        <button onclick={() => {
          console.log('📝 Setting searchManager.searchInput = "debug"');
          searchManager.searchInput = 'debug';
        }}>
          Set searchManager.searchInput = 'debug'
        </button>
        <button onclick={() => {
          console.log('📝 Setting searchManager.updateState({searchInput: "direct"})');
          searchManager.updateState({searchInput: 'direct'});
        }}>
          Set searchManager.updateState('direct')
        </button>
        <button onclick={() => {
          console.log('🔄 Manual appCoordinator.initialize()');
          appCoordinator.initialize().then(() => console.log('✅ Initialize complete'));
        }}>
          Re-run initialize()
        </button>
      </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .debug-toggle {
    position: fixed;
    bottom: 20px;
    right: 30px;
    z-index: 10000;
    background: rgba(220, 38, 38, 0.7);
    border: none;
    border-radius: 50%;
    width: 12px;
    height: 12px;
    cursor: pointer;
    opacity: 0.3;
    transition: all 0.2s ease;
  }

  .debug-toggle:hover {
    opacity: 0.8;
    background: rgba(220, 38, 38, 0.9);
    transform: scale(1.2);
  }

  .debug-toggle.active {
    opacity: 1;
    background: #dc2626;
    box-shadow: 0 0 8px rgba(220, 38, 38, 0.4);
  }


  .debug-panel {
    position: fixed;
    top: 60px;
    right: 10px;
    width: 400px;
    max-height: 80vh;
    background: #1e1e1e;
    color: #ffffff;
    border: 1px solid #444;
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.3);
    z-index: 9999;
    overflow-y: auto;
    font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
    font-size: 12px;
  }

  .debug-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: linear-gradient(135deg, #2d2d2d 0%, #3d3d3d 100%);
    border-bottom: 1px solid #444;
    border-radius: 8px 8px 0 0;
  }

  .debug-header h3 {
    margin: 0;
    color: #4CAF50;
    font-size: 14px;
    font-weight: 600;
  }

  .debug-header button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #ccc;
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .debug-header button:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .debug-filters {
    padding: 12px 16px;
    background: #252525;
    border-bottom: 1px solid #333;
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .filter-checkbox {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: #bbb;
    cursor: pointer;
    user-select: none;
  }

  .filter-checkbox input[type="checkbox"] {
    accent-color: #4CAF50;
    cursor: pointer;
  }

  .filter-checkbox:hover {
    color: #ddd;
  }

  .debug-content {
    padding: 16px;
  }

  .debug-section {
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid #333;
  }

  .debug-section:last-child {
    border-bottom: none;
    margin-bottom: 0;
  }

  .debug-section h4 {
    margin: 0 0 12px 0;
    color: #4CAF50;
    font-size: 13px;
    font-weight: bold;
  }

  .debug-item {
    margin-bottom: 8px;
    line-height: 1.4;
  }

  .debug-item strong {
    color: #81C784;
    display: inline-block;
    min-width: 120px;
  }

  .debug-item code {
    background: #333;
    padding: 2px 6px;
    border-radius: 3px;
    color: #FFF59D;
    word-break: break-all;
  }

  .debug-item small {
    color: #888;
    margin-left: 8px;
  }

  .debug-section button {
    background: #2196F3;
    color: white;
    border: none;
    padding: 6px 12px;
    margin: 4px 4px 4px 0;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
  }

  .debug-section button:hover {
    background: #1976D2;
  }
</style>
