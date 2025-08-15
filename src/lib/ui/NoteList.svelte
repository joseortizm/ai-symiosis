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
    border-bottom: 2px solid #504945;
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
    color: #928374;
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
    border-bottom: 1px solid #3c3836;
    background: none;
    color: #ebdbb2;
    text-align: left;
    font-size: 0.95em;
    transition: background-color 0.1s ease;
    contain: layout;
  }
  .selected {
    background-color: #504945 !important;
    color: #fe8019;
  }
  .notes-list::-webkit-scrollbar {
    width: 8px;
  }
  .notes-list::-webkit-scrollbar-track {
    background: #282828;
  }
  .notes-list::-webkit-scrollbar-thumb {
    background: #504945;
    border-radius: 4px;
  }
</style>
