<!--
UI Layer - Debug Panel
Development tool for inspecting application state and manager values.
Shows reactive state from managers and services for debugging.
-->

<script lang="ts">
import { getContext } from 'svelte';
import type { createSearchManager } from '../core/searchManager.svelte';
import type { createAppCoordinator } from '../app/appCoordinator.svelte';
import type { createEditorManager } from '../core/editorManager.svelte';
import type { createFocusManager } from '../core/focusManager.svelte';
import { noteService } from '../services/noteService.svelte';
import { configService } from '../services/configService.svelte';

const { searchManager, appCoordinator, editorManager, focusManager } = getContext<{
  searchManager: ReturnType<typeof createSearchManager>;
  appCoordinator: ReturnType<typeof createAppCoordinator>;
  editorManager: ReturnType<typeof createEditorManager>;
  focusManager: ReturnType<typeof createFocusManager>;
}>('managers');

const context = appCoordinator.context;

// Debug panel visibility and configuration
let isVisible = $state(false);
let isEnabled = $state(true);

// Filter toggles for different sections
let showServices = $state(true);
let showManagers = $state(true);
let showAppCoordinator = $state(true);
let showDialogs = $state(true);
let showContent = $state(true);

function togglePanel() {
  isVisible = !isVisible;
}

function formatValue(value: any): string {
  if (value === null) return 'null';
  if (value === undefined) return 'undefined';
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'string') return `"${value}"`;
  if (Array.isArray(value)) {
    if (value.length === 0) return '[]';
    if (value.length <= 3) return JSON.stringify(value);
    return `[${value.slice(0, 2).map(v => JSON.stringify(v)).join(', ')}, ...] (${value.length} total)`;
  }
  if (typeof value === 'object') {
    const keys = Object.keys(value);
    if (keys.length === 0) return '{}';
    if (keys.length <= 3) return JSON.stringify(value);
    return `{${keys.slice(0, 2).join(', ')}, ...} (${keys.length} keys)`;
  }
  return String(value);
}

// Keyboard shortcut support - handle at capture phase for highest precedence
function handleKeydown(event: KeyboardEvent) {
  if (isEnabled && event.metaKey &&  event.altKey && event.key === 'd') {
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();
    togglePanel();
  }
}
</script>

<!-- Debug toggle button - small subtle red circle -->
<!-- svelte-ignore a11y_consider_explicit_label -->
<button
  class="debug-toggle"
  onclick={togglePanel}
  title="Debug Panel (Cmd+Alt+D)"
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
        <input type="checkbox" bind:checked={showServices} />
        Services
      </label>
      <label class="filter-checkbox">
        <input type="checkbox" bind:checked={showManagers} />
        Managers
      </label>
      <label class="filter-checkbox">
        <input type="checkbox" bind:checked={showAppCoordinator} />
        AppCoordinator
      </label>
      <label class="filter-checkbox">
        <input type="checkbox" bind:checked={showDialogs} />
        Dialogs
      </label>
      <label class="filter-checkbox">
        <input type="checkbox" bind:checked={showContent} />
        Content
      </label>
    </div>

    <div class="debug-content">
      {#if showServices}
      <div class="debug-section">
        <h4>🛠️ Services</h4>
        <div class="debug-subsection">
          <h5>NoteService</h5>
          <div class="debug-item">
            <strong>isLoading:</strong>
            <code>{formatValue(noteService.isLoading)}</code>
          </div>
          <div class="debug-item">
            <strong>error:</strong>
            <code>{formatValue(noteService.error)}</code>
          </div>
          <div class="debug-item">
            <strong>lastOperation:</strong>
            <code>{formatValue(noteService.lastOperation)}</code>
          </div>
        </div>
        <div class="debug-subsection">
          <h5>ConfigService</h5>
          <div class="debug-item">
            <strong>isVisible:</strong>
            <code>{formatValue(configService.isVisible)}</code>
          </div>
          <div class="debug-item">
            <strong>isLoading:</strong>
            <code>{formatValue(configService.isLoading)}</code>
          </div>
          <div class="debug-item">
            <strong>error:</strong>
            <code>{formatValue(configService.error)}</code>
          </div>
          <div class="debug-item">
            <strong>content length:</strong>
            <code>{formatValue(configService.content?.length || 0)} chars</code>
          </div>
        </div>
      </div>
      {/if}

      {#if showManagers}
      <div class="debug-section">
        <h4>⚙️ Managers</h4>
        <div class="debug-subsection">
          <h5>SearchManager</h5>
          <div class="debug-item">
            <strong>searchInput:</strong>
            <code>{formatValue(searchManager.searchInput)}</code>
          </div>
          <div class="debug-item">
            <strong>filteredNotes:</strong>
            <code>{formatValue(searchManager.filteredNotes)}</code>
          </div>
          <div class="debug-item">
            <strong>isLoading:</strong>
            <code>{formatValue(searchManager.isLoading)}</code>
          </div>
        </div>
        <div class="debug-subsection">
          <h5>EditorManager</h5>
          <div class="debug-item">
            <strong>isEditMode:</strong>
            <code>{formatValue(editorManager.isEditMode)}</code>
          </div>
          <div class="debug-item">
            <strong>isDirty:</strong>
            <code>{formatValue(editorManager.isDirty)}</code>
          </div>
          <div class="debug-item">
            <strong>nearestHeaderText:</strong>
            <code>{formatValue(editorManager.nearestHeaderText)}</code>
          </div>
          <div class="debug-item">
            <strong>editContent length:</strong>
            <code>{formatValue(editorManager.editContent?.length || 0)} chars</code>
          </div>
        </div>
        <div class="debug-subsection">
          <h5>FocusManager</h5>
          <div class="debug-item">
            <strong>isSearchInputFocused:</strong>
            <code>{formatValue(focusManager.isSearchInputFocused)}</code>
          </div>
          <div class="debug-item">
            <strong>isNoteContentFocused:</strong>
            <code>{formatValue(focusManager.isNoteContentFocused)}</code>
          </div>
          <div class="debug-item">
            <strong>searchElement:</strong>
            <code>{formatValue(focusManager.searchElement ? 'HTMLInputElement' : 'null')}</code>
          </div>
          <div class="debug-item">
            <strong>noteContentElement:</strong>
            <code>{formatValue(focusManager.noteContentElement ? 'HTMLElement' : 'null')}</code>
          </div>
          <div class="debug-item">
            <strong>noteListElement:</strong>
            <code>{formatValue(focusManager.noteListElement ? 'HTMLElement' : 'null')}</code>
          </div>
        </div>
      </div>
      {/if}

      {#if showAppCoordinator}
      <div class="debug-section">
        <h4>🎯 AppCoordinator</h4>
        <div class="debug-item">
          <strong>query:</strong>
          <code>{formatValue(appCoordinator.query)}</code>
        </div>
        <div class="debug-item">
          <strong>selectedNote:</strong>
          <code>{formatValue(appCoordinator.selectedNote)}</code>
        </div>
        <div class="debug-item">
          <strong>selectedIndex:</strong>
          <code>{formatValue(appCoordinator.selectedIndex)}</code>
        </div>
        <div class="debug-item">
          <strong>filteredNotes:</strong>
          <code>{formatValue(appCoordinator.filteredNotes)}</code>
        </div>
        <div class="debug-item">
          <strong>isLoading:</strong>
          <code>{formatValue(appCoordinator.isLoading)}</code>
        </div>
      </div>
      {/if}

      {#if showDialogs}
      <div class="debug-section">
        <h4>💬 Dialogs</h4>
        <div class="debug-item">
          <strong>showCreateDialog:</strong>
          <code>{formatValue(context.dialogManager.showCreateDialog)}</code>
        </div>
        <div class="debug-item">
          <strong>showRenameDialog:</strong>
          <code>{formatValue(context.dialogManager.showRenameDialog)}</code>
        </div>
        <div class="debug-item">
          <strong>showDeleteDialog:</strong>
          <code>{formatValue(context.dialogManager.showDeleteDialog)}</code>
        </div>
        <div class="debug-item">
          <strong>showUnsavedChangesDialog:</strong>
          <code>{formatValue(context.dialogManager.showUnsavedChangesDialog)}</code>
        </div>
        <div class="debug-item">
          <strong>newNoteName:</strong>
          <code>{formatValue(context.dialogManager.newNoteName)}</code>
        </div>
        <div class="debug-item">
          <strong>newNoteNameForRename:</strong>
          <code>{formatValue(context.dialogManager.newNoteNameForRename)}</code>
        </div>
        <div class="debug-item">
          <strong>deleteKeyPressCount:</strong>
          <code>{formatValue(context.dialogManager.deleteKeyPressCount)}</code>
        </div>
      </div>
      {/if}

      {#if showContent}
      <div class="debug-section">
        <h4>📄 Content</h4>
        <div class="debug-item">
          <strong>noteContent length:</strong>
          <code>{formatValue(context.contentManager.noteContent?.length || 0)} chars</code>
        </div>
        <div class="debug-item">
          <strong>highlightedContent length:</strong>
          <code>{formatValue(context.contentManager.highlightedContent?.length || 0)} chars</code>
        </div>
        <div class="debug-item">
          <strong>areHighlightsCleared:</strong>
          <code>{formatValue(context.contentManager.areHighlightsCleared)}</code>
        </div>
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

  .debug-subsection {
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid #444;
  }

  .debug-subsection:last-child {
    border-bottom: none;
    margin-bottom: 0;
  }

  .debug-subsection h5 {
    margin: 0 0 8px 0;
    color: #81C784;
    font-size: 12px;
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

</style>
