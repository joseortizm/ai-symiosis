<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  
  let filteredNotes = $state([]);
  let selectedNote = $state(null);
  let selectedIndex = $state(-1);
  let searchInput = $state('');
  let noteContent = $state('');
  let searchElement;
  let noteListElement = $state();
  let isSearchInputFocused = $state(false);
  let isLoading = $state(false);

  onMount(() => {
    searchElement.focus();
    // Load initial notes
    loadNotes('');
  });

  // Debounced search to improve performance
  let searchTimeout;
  function debounceSearch(query) {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      loadNotes(query);
    }, 150);
  }

  async function loadNotes(query) {
    try {
      isLoading = true;
      filteredNotes = await invoke("list_notes", { query });
      
      // Reset selection when notes change
      if (filteredNotes.length === 0) {
        selectedIndex = -1;
      } else if (selectedIndex >= filteredNotes.length) {
        selectedIndex = filteredNotes.length - 1;
      } else if (selectedIndex === -1 && filteredNotes.length > 0) {
        selectedIndex = 0;
      }
    } catch (e) {
      console.error('Failed to load notes:', e);
      filteredNotes = [];
    } finally {
      isLoading = false;
    }
  }

  // Watch search input changes
  $effect(() => {
    debounceSearch(searchInput);
  });

  // Update selected note when index changes
  $effect(() => {
    if (filteredNotes.length > 0 && selectedIndex !== -1) {
      selectedNote = filteredNotes[selectedIndex];
    } else {
      selectedNote = null;
    }
  });

  // Load note content when selected note changes
  $effect(async () => {
    if (selectedNote) {
      try {
        noteContent = await invoke("get_note_content", { noteName: selectedNote });
      } catch (e) {
        console.error("Failed to load note content:", e);
        noteContent = `Error loading note: ${e}`;
      }
    } else {
      noteContent = '';
    }
  });

  function selectNote(note, index) {
    selectedIndex = index;
  }

  function handleKeydown(event) {
    if (isSearchInputFocused) {
      switch (event.key) {
        case 'Enter':
          event.preventDefault();
          if (filteredNotes.length > 0) {
            selectedIndex = 0;
            // Open the selected note
            if (selectedNote) {
              invoke('open_note', { noteName: selectedNote });
            }
          }
          return;
        case 'ArrowUp':
          if (event.ctrlKey) {
            event.preventDefault();
            selectedIndex = Math.max(0, selectedIndex - 1);
            return;
          }
          break;
        case 'ArrowDown':
          if (event.ctrlKey) {
            event.preventDefault();
            selectedIndex = Math.min(filteredNotes.length - 1, selectedIndex + 1);
            return;
          }
          break;
        case 'p':
          if (event.ctrlKey) {
            event.preventDefault();
            selectedIndex = Math.max(0, selectedIndex - 1);
            return;
          }
          break;
        case 'n':
          if (event.ctrlKey) {
            event.preventDefault();
            selectedIndex = Math.min(filteredNotes.length - 1, selectedIndex + 1);
            return;
          }
          break;
      }
    }

    if (filteredNotes.length === 0) return;

    switch (event.key) {
      case 'ArrowUp':
      case 'k':
        event.preventDefault();
        selectedIndex = Math.max(0, selectedIndex - 1);
        break;
      case 'ArrowDown':
      case 'j':
        event.preventDefault();
        selectedIndex = Math.min(filteredNotes.length - 1, selectedIndex + 1);
        break;
      case 'Enter':
        if (selectedNote) {
          invoke('open_note', { noteName: selectedNote });
        }
        break;
      case 'Escape':
        searchElement.focus();
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="container">
  <input
    type="text"
    bind:value={searchInput}
    placeholder="Search notes..."
    class="search-input"
    bind:this={searchElement}
    onfocus={() => isSearchInputFocused = true}
    onblur={() => isSearchInputFocused = false}
  >

  <div class="notes-list-container">
    <div class="notes-list">
      {#if isLoading}
        <div class="loading">Loading...</div>
      {:else if filteredNotes.length === 0}
        <div class="no-notes">No notes found</div>
      {:else}
        <ul bind:this={noteListElement} tabindex="-1">
          {#each filteredNotes as note, index}
            <li>
              <button 
                class:selected={index === selectedIndex} 
                onclick={() => selectNote(note, index)}
              >
                {note}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>

  <div class="note-preview">
    {#if selectedNote}
      <div class="note-header">
        <h3>{selectedNote}</h3>
      </div>
      <div class="note-content">
        <pre>{noteContent}</pre>
      </div>
    {:else}
      <div class="no-selection">
        <p>Select a note to preview its content</p>
      </div>
    {/if}
  </div>
</main>

<style>
  .container {
    margin: 0;
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #282828;
    color: #ebdbb2;
    font-family: 'Inter', sans-serif;
  }

  .search-input {
    background-color: #3c3836;
    color: #ebdbb2;
    border: 1px solid #504945;
    border-radius: 8px;
    font-size: 1.2em;
    padding: 0.6em;
    margin: 0.5em;
    flex-shrink: 0;
  }

  .search-input:focus {
    outline: none;
    border-color: #83a598;
    box-shadow: 0 0 0 2px rgba(131, 165, 152, 0.2);
  }

  .notes-list-container {
    flex: 1;
    min-height: 0;
    border-bottom: 2px solid #504945;
  }

  .notes-list {
    height: 100%;
    overflow-y: auto;
  }

  .loading, .no-notes {
    padding: 2em;
    text-align: center;
    color: #928374;
    font-style: italic;
  }

  .note-preview {
    flex: 1.2;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .note-header {
    padding: 0.8em 1em;
    border-bottom: 1px solid #504945;
    background-color: #3c3836;
    flex-shrink: 0;
  }

  .note-header h3 {
    margin: 0;
    color: #fe8019;
    font-size: 1.1em;
    font-weight: 500;
  }

  .note-content {
    flex: 1;
    padding: 1em;
    overflow-y: auto;
  }

  .note-content pre {
    margin: 0;
    white-space: pre-wrap;
    word-wrap: break-word;
    font-family: 'Inter', sans-serif;
    font-size: 0.9em;
    line-height: 1.6;
    color: #ebdbb2;
  }

  .no-selection {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #928374;
    font-style: italic;
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  li {
    margin: 0;
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
  }

  button:hover {
    background-color: #3c3836;
  }

  .selected {
    background-color: #504945 !important;
    color: #fe8019;
  }

  /* Custom scrollbar for notes list */
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

  /* Custom scrollbar for note content */
  .note-content::-webkit-scrollbar {
    width: 8px;
  }

  .note-content::-webkit-scrollbar-track {
    background: #282828;
  }

  .note-content::-webkit-scrollbar-thumb {
    background: #504945;
    border-radius: 4px;
  }

  .note-content::-webkit-scrollbar-thumb:hover {
    background: #665c54;
  }
</style>
