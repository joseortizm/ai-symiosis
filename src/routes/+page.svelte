<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  let filteredNotes = $state([]);
  let selectedNote = $state(null);
  let selectedIndex = $state(-1);
  let searchInput = $state('');
  let noteContent = $state('');
  let searchElement;
  let noteListElement;
  let isSearchInputFocused = $state(false);

  onMount(() => {
    searchElement.focus();
  });

  $effect(async () => {
    try {
      filteredNotes = await invoke("list_notes", { query: searchInput });
      if (filteredNotes.length === 0) {
        selectedIndex = -1;
      } else if (selectedIndex >= filteredNotes.length) {
        selectedIndex = filteredNotes.length - 1;
      } else if (selectedIndex === -1 && filteredNotes.length > 0) {
        selectedIndex = 0;
      }
    } catch (e) {
      console.error(e);
    }
  });

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
        noteContent = "Failed to load note content.";
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
            noteListElement.focus();
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
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="container">
  <input
    type="text"
    bind:value={searchInput}
    placeholder="Search..."
    class="search-input"
    bind:this={searchElement}
    onfocus={() => isSearchInputFocused = true}
    onblur={() => isSearchInputFocused = false}
  >

  <div class="content-area">
    <div class="notes-list">
      <ul bind:this={noteListElement} tabindex="-1">
        {#each filteredNotes as note, index}
          <li>
            <button class:selected={note === selectedNote} onclick={() => selectNote(note, index)}>
              {note}
            </button>
          </li>
        {/each}
      </ul>
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
    font-size: 1.5em;
    padding: 0.8em;
    margin: 0.5em;
  }

  .search-input:focus {
    outline: none;
    border-color: #83a598;
  }

  .content-area {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .notes-list {
    flex: 0 0 40%;
    border-right: 1px solid #504945;
    overflow-y: auto;
  }

  .note-preview {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .note-header {
    padding: 1em;
    border-bottom: 1px solid #504945;
    background-color: #3c3836;
  }

  .note-header h3 {
    margin: 0;
    color: #fe8019;
    font-size: 1.2em;
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
    font-size: 0.95em;
    line-height: 1.6;
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
    padding: 0.8em 1.2em;
    cursor: pointer;
    border: none;
    border-bottom: 1px solid #3c3836;
    background: none;
    color: #ebdbb2;
    text-align: left;
    font-size: 1em;
  }

  button:hover {
    background-color: #3c3836;
  }

  .selected {
    background-color: #504945;
    color: #fe8019;
  }
</style>
