<!--
UI Layer - Note List
Displays filtered notes with virtualization and selection highlighting.
Handles note selection state and integrates with keyboard navigation.
-->

<script lang="ts">
  // Imports
  import { getContext } from 'svelte'
  import type {
    AppManagers,
    AppState,
    AppActions,
  } from '../app/appCoordinator.svelte'

  // Context and state
  const { focusManager } = getContext<AppManagers>('managers')
  const appState = getContext<AppState>('state')
  const actions = getContext<AppActions>('actions')

  let noteListElement = $state<HTMLElement | undefined>(undefined)

  // Effects
  $effect(() => {
    if (noteListElement) {
      focusManager.setNoteListElement(noteListElement)
    }
  })
</script>

<div class="notes-list-container">
  <div class="notes-list">
    {#if appState.isLoading && appState.filteredNotes.length === 0}
      <div class="loading">Loading...</div>
    {:else if appState.filteredNotes.length === 0}
      <div class="no-notes">No notes found</div>
    {:else}
      <ul bind:this={noteListElement} tabindex="-1">
        {#each appState.filteredNotes as note, index (note)}
          <li>
            <button
              class:selected={index === focusManager.selectedIndex}
              tabindex="-1"
              onclick={() => {
                focusManager.setSelectedIndex(index)
                actions.loadNoteContent(note)
              }}
            >
              {note}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .notes-list-container {
    flex: 0.4;
    min-height: 0;
    border-bottom: 2px solid var(--theme-border);
  }
  .notes-list {
    height: 100%;
    overflow-y: auto;
    transform: translateZ(0);
    will-change: scroll-position;
    scroll-behavior: smooth;
  }
  .loading,
  .no-notes {
    padding: 2em;
    text-align: center;
    color: var(--theme-text-muted);
    font-style: italic;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    contain: content;
  }
  li {
    margin: 0;
    contain: layout;
  }
  button {
    width: 100%;
    padding: 0.6em 1em;
    cursor: pointer;
    border: none;
    border-bottom: 1px solid var(--theme-bg-secondary);
    background: none;
    color: var(--theme-text-primary);
    text-align: left;
    font-size: 0.95em;
    contain: layout;
  }
  .selected {
    background-color: var(--theme-bg-tertiary) !important;
    color: var(--theme-highlight);
  }
  .notes-list::-webkit-scrollbar {
    width: 8px;
  }
  .notes-list::-webkit-scrollbar-track {
    background: var(--theme-bg-primary);
  }
  .notes-list::-webkit-scrollbar-thumb {
    background: var(--theme-border);
    border-radius: 4px;
  }
</style>
