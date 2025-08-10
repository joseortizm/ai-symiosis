<script lang="ts">
  import { getContext } from 'svelte';

  const { appCoordinator } = getContext('managers');

  let noteListElement = $state<HTMLElement | undefined>(undefined);

  $effect(() => {
    if (noteListElement) {
      appCoordinator.context.focusManager.setNoteListElement(noteListElement);
    }
  });
</script>

<div class="notes-list-container">
  <div class="notes-list">
    {#if appCoordinator.context.state.isLoading && appCoordinator.context.state.filteredNotes.length === 0}
      <div class="loading">Loading...</div>
    {:else if appCoordinator.context.state.filteredNotes.length === 0}
      <div class="no-notes">No notes found</div>
    {:else}
      <ul bind:this={noteListElement} tabindex="-1">
        {#each appCoordinator.context.state.filteredNotes as note, index (note)}
          <li>
            <button
              class:selected={index === appCoordinator.context.state.selectedIndex}
              tabindex="-1"
              onclick={() => appCoordinator.context.selectNote(note, index)}
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
.loading, .no-notes {
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
button:hover {
  background-color: #3c3836;
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
.notes-list::-webkit-scrollbar-thumb:hover {
  background: #665c54;
}
</style>
