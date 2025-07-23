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
let noteContentElement = $state();
let isSearchInputFocused = $state(false);
let isNoteContentFocused = $state(false);
let isLoading = $state(false);
let lastQuery = $state('');
let highlightedContent = $state('');

// Edit mode state
let isEditMode = $state(false);
let editContent = $state('');
let editTextarea = $state();

// Performance optimizations
let searchAbortController = null;
let contentAbortController = null;

onMount(() => {
  searchElement.focus();
  // Load initial notes
  loadNotesImmediate('');
});

// Function to highlight search terms in content
function highlightSearchTerms(content, query) {
  if (!query.trim()) {
    return content;
  }

  // Escape special regex characters in the query
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escapedQuery})`, 'gi');

  // Replace matches with highlighted spans
  return content.replace(regex, '<mark class="highlight">$1</mark>');
}

// Function to scroll to first search match in note content
function scrollToFirstMatch() {
  if (noteContentElement && lastQuery.trim()) {
    // Use setTimeout to ensure DOM is updated
    setTimeout(() => {
      const firstMatch = noteContentElement.querySelector('.highlight');
      if (firstMatch) {
        firstMatch.scrollIntoView({
          behavior: 'smooth',
          block: 'center'
        });
      }
    }, 100);
  }
}

function scrollToSelected() {
  if (noteListElement && selectedIndex >= 0) {
    const selectedButton = noteListElement.children[selectedIndex]?.querySelector('button');
    if (selectedButton) {
      selectedButton.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest'
      });
    }
  }
}

// Optimized debounced search - only trigger if query actually changed
let searchTimeout;
function debounceSearch(query) {
  // Don't search if query hasn't changed
  if (query === lastQuery) return;

  clearTimeout(searchTimeout);

  // Cancel any pending search requests
  if (searchAbortController) {
    searchAbortController.abort();
  }

  searchTimeout = setTimeout(() => {
    loadNotesImmediate(query);
  }, 100); // Reduced debounce time for snappier feel
}

async function loadNotesImmediate(query) {
  // Cancel any pending requests
  if (searchAbortController) {
    searchAbortController.abort();
  }

  searchAbortController = new AbortController();
  const currentController = searchAbortController;

  try {
    isLoading = true;
    lastQuery = query;

    const newNotes = await invoke("list_notes", { query });

    // Check if this request was cancelled
    if (currentController.signal.aborted) {
      return;
    }

    // Only update if notes actually changed to prevent flashing
    if (JSON.stringify(newNotes) !== JSON.stringify(filteredNotes)) {
      filteredNotes = newNotes;

      // Reset selection to top when search results change
      if (newNotes.length === 0) {
        selectedIndex = -1;
      } else {
        selectedIndex = 0; // Always select first item
      }
    }
  } catch (e) {
    if (!currentController.signal.aborted) {
      console.error('Failed to load notes:', e);
      filteredNotes = [];
      selectedIndex = -1;
    }
  } finally {
    if (!currentController.signal.aborted) {
      isLoading = false;
    }
  }
}

// Reactive search with optimization
$effect(() => {
  debounceSearch(searchInput);
});

// Update selected note when index changes (with caching)
$effect(() => {
  const newSelectedNote = filteredNotes.length > 0 && selectedIndex !== -1
    ? filteredNotes[selectedIndex]
    : null;

  // Only update if actually changed
  if (newSelectedNote !== selectedNote) {
    selectedNote = newSelectedNote;
    // Exit edit mode when switching notes
    isEditMode = false;
  }
});

// Scroll to selected item when selection changes
$effect(() => {
  if (selectedIndex >= 0) {
    // Use requestAnimationFrame to ensure DOM is updated
    requestAnimationFrame(() => {
      scrollToSelected();
    });
  }
});

// Optimized note content loading with caching and abort control
$effect(async () => {
  if (!selectedNote) {
    noteContent = '';
    highlightedContent = '';
    return;
  }

  // Cancel any pending content requests
  if (contentAbortController) {
    contentAbortController.abort();
  }

  contentAbortController = new AbortController();
  const currentController = contentAbortController;

  try {
    const content = await invoke("get_note_content", { noteName: selectedNote });

    // Check if this request was cancelled
    if (!currentController.signal.aborted) {
      noteContent = content;
      highlightedContent = highlightSearchTerms(content, lastQuery);
      // Scroll to first match after content is loaded
      requestAnimationFrame(() => {
        scrollToFirstMatch();
      });
    }
  } catch (e) {
    if (!currentController.signal.aborted) {
      console.error("Failed to load note content:", e);
      noteContent = `Error loading note: ${e}`;
      highlightedContent = noteContent;
    }
  }
});

function selectNote(note, index) {
  if (selectedIndex !== index) {
    selectedIndex = index;
  }
}

async function enterEditMode() {
  if (selectedNote) {
    try {
      // Get raw content for editing (not rendered HTML)
      const rawContent = await invoke("get_note_raw_content", { noteName: selectedNote });
      isEditMode = true;
      editContent = rawContent;
      
      // Focus on textarea after it's rendered
      requestAnimationFrame(() => {
        if (editTextarea) {
          editTextarea.focus();
        }
      });
    } catch (e) {
      console.error("Failed to load raw note content:", e);
    }
  }
}

function exitEditMode() {
  isEditMode = false;
  // You might want to save changes here
  // For now, just exit edit mode
}

async function saveNote() {
  if (!selectedNote || !editContent) return;
  
  try {
    // You'll need to implement a save_note command in your Rust backend
    await invoke("save_note", { 
      noteName: selectedNote, 
      content: editContent 
    });
    
    // Refresh the note content after saving
    const content = await invoke("get_note_content", { noteName: selectedNote });
    noteContent = content;
    highlightedContent = highlightSearchTerms(content, lastQuery);
    
    isEditMode = false;
  } catch (e) {
    console.error("Failed to save note:", e);
  }
}

function handleKeydown(event) {
  if (isSearchInputFocused) {
    switch (event.key) {
      case 'Enter':
        event.preventDefault();
        if (filteredNotes.length > 0 && selectedNote) {
          // Enter edit mode directly
          enterEditMode();
        }
        return;
      case 'o':
        if (event.ctrlKey) {
          event.preventDefault();
          if (selectedNote) {
            invoke("open_note", { noteName: selectedNote });
          }
          return;
        }
        break;
      case 'u':
        if (event.ctrlKey) {
          event.preventDefault();
          if (noteContentElement) {
            noteContentElement.scrollBy({ top: -200, behavior: 'smooth' });
          }
          return;
        }
        break;
      case 'd':
        if (event.ctrlKey) {
          event.preventDefault();
          if (noteContentElement) {
            noteContentElement.scrollBy({ top: 200, behavior: 'smooth' });
          }
          return;
        }
        break;
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

  // Handle edit mode
  if (isEditMode) {
    switch (event.key) {
      case 'Escape':
        event.preventDefault();
        exitEditMode();
        searchElement.focus();
        return;
      case 's':
        if (event.ctrlKey) {
          event.preventDefault();
          saveNote();
          return;
        }
        break;
    }
  }

  // Handle note content navigation
  if (isNoteContentFocused && !isEditMode) {
    switch (event.key) {
      case 'ArrowUp':
      case 'k':
        event.preventDefault();
        noteContentElement.scrollBy({ top: -50, behavior: 'smooth' });
        return;
      case 'ArrowDown':
      case 'j':
        event.preventDefault();
        noteContentElement.scrollBy({ top: 50, behavior: 'smooth' });
        return;
      case 'p':
        if (event.ctrlKey) {
          event.preventDefault();
          noteContentElement.scrollBy({ top: -50, behavior: 'smooth' });
          return;
        }
        break;
      case 'n':
        if (event.ctrlKey) {
          event.preventDefault();
          noteContentElement.scrollBy({ top: 50, behavior: 'smooth' });
          return;
        }
        break;
      case 'Escape':
        event.preventDefault();
        searchElement.focus();
        return;
      case 'e':
        event.preventDefault();
        enterEditMode();
        return;
    }
  }

  if (filteredNotes.length === 0) return;

  // Global navigation when not in search or note content
  if (!isSearchInputFocused && !isNoteContentFocused && !isEditMode) {
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
          enterEditMode();
        }
        break;
      case 'Escape':
        searchElement.focus();
        break;
    }
  }
}

// Clean up on component destroy
onMount(() => {
  return () => {
    if (searchAbortController) searchAbortController.abort();
    if (contentAbortController) contentAbortController.abort();
    clearTimeout(searchTimeout);
  };
});
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="container">
  <input
    type="text"
    bind:value={searchInput}
    placeholder="Search notes... (Enter: edit, Ctrl+O: open, Ctrl+U/D: scroll)"
    class="search-input"
    bind:this={searchElement}
    onfocus={() => isSearchInputFocused = true}
    onblur={() => isSearchInputFocused = false}
  >

  <div class="notes-list-container">
    <div class="notes-list">
      {#if isLoading && filteredNotes.length === 0}
        <div class="loading">Loading...</div>
      {:else if filteredNotes.length === 0}
        <div class="no-notes">No notes found</div>
      {:else}
        <ul bind:this={noteListElement} tabindex="-1">
          {#each filteredNotes as note, index (note)}
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
      {#if isEditMode}
        <div class="edit-mode">
          <div class="edit-header">
            <h3>Editing: {selectedNote}</h3>
            <div class="edit-controls">
              <button onclick={saveNote} class="save-btn">Save (Ctrl+S)</button>
              <button onclick={exitEditMode} class="cancel-btn">Cancel (Esc)</button>
            </div>
          </div>
          <textarea
            bind:value={editContent}
            bind:this={editTextarea}
            class="edit-textarea"
            placeholder="Start typing..."
          ></textarea>
        </div>
      {:else}
        <div class="note-content"
          bind:this={noteContentElement}
          tabindex="0"
          onfocus={() => isNoteContentFocused = true}
          onblur={() => isNoteContentFocused = false}
          onclick={(event) => {
            const target = event.target;
            if (target.tagName === 'A') {
              event.preventDefault();
              invoke("open_external_link", { url: target.href });
            }
          }}>
          <div class="note-text">{@html highlightedContent}</div>
        </div>
      {/if}
    {:else}
      <div class="no-selection">
        <p>Select a note to preview its content</p>
        <p class="help-text">Press Enter to edit, E to edit when focused, Ctrl+O to open externally</p>
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
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
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

.note-preview {
  flex: 1.2;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.edit-mode {
  flex: 1;
  display: flex;
  flex-direction: column;
  background-color: #32302f;
}

.edit-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.8em 1em;
  border-bottom: 1px solid #504945;
  background-color: #3c3836;
  flex-shrink: 0;
}

.edit-header h3 {
  margin: 0;
  color: #fe8019;
  font-size: 1.1em;
  font-weight: 500;
}

.edit-controls {
  display: flex;
  gap: 0.5em;
}

.save-btn, .cancel-btn {
  padding: 0.4em 0.8em;
  border: none;
  border-radius: 4px;
  font-size: 0.9em;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.save-btn {
  background-color: #b8bb26;
  color: #282828;
}

.save-btn:hover {
  background-color: #98971a;
}

.cancel-btn {
  background-color: #504945;
  color: #ebdbb2;
}

.cancel-btn:hover {
  background-color: #665c54;
}

.edit-textarea {
  flex: 1;
  padding: 1em;
  background-color: #32302f;
  color: #fbf1c7;
  border: none;
  outline: none;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.95em;
  line-height: 1.6;
  resize: none;
}

.note-content {
  flex: 1;
  padding: 1em;
  overflow-y: auto;
  transform: translateZ(0);
  will-change: scroll-position;
  outline: none;
  border: 2px solid transparent;
  transition: border-color 0.2s ease;
  background-color: #32302f;
}

.note-content:focus {
  border-color: #83a598;
}

.note-text {
  color: #fbf1c7;
  font-family: 'Inter', sans-serif;
  font-size: 0.95em;
  line-height: 1.6;
  white-space: normal;
}

.note-text h1,
.note-text h2,
.note-text h3,
.note-text h4 {
  margin: 1em 0 0.5em;
  font-weight: bold;
  color: #fabd2f;
}

.note-text h1 { font-size: 1.5em; }
.note-text h2 { font-size: 1.3em; }
.note-text h3 { font-size: 1.15em; }

.note-text p {
  margin: 0.5em 0;
}

.note-text a {
  color: #83a598;
  text-decoration: underline;
  word-break: break-word;
}

.note-text a:hover {
  color: #b8bb26;
}

.note-text code {
  background: #3c3836;
  padding: 0.2em 0.4em;
  border-radius: 4px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.95em;
  color: #d3869b;
}

.note-text pre {
  background: #3c3836;
  padding: 1em;
  overflow-x: auto;
  border-radius: 6px;
  font-family: 'JetBrains Mono', monospace;
  color: #fbf1c7;
  margin: 1em 0;
  font-size: 0.9em;
}

.note-text ul,
.note-text ol {
  margin: 0.5em 0 0.5em 1.2em;
  padding-left: 1em;
}

.note-text blockquote {
  margin: 1em 0;
  padding-left: 1em;
  border-left: 3px solid #504945;
  color: #d5c4a1;
  font-style: italic;
}

.note-text hr {
  border: none;
  border-top: 1px solid #504945;
  margin: 1em 0;
}

.highlight {
  background-color: #fabd2f;
  color: #282828;
  padding: 0.1em 0.2em;
  border-radius: 3px;
  font-weight: 500;
}

.no-selection {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #928374;
  font-style: italic;
  text-align: center;
}

.help-text {
  font-size: 0.9em;
  margin-top: 0.5em;
  color: #665c54;
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

/* Custom scrollbar optimizations */
.notes-list::-webkit-scrollbar, .note-content::-webkit-scrollbar, .edit-textarea::-webkit-scrollbar {
  width: 8px;
}

.notes-list::-webkit-scrollbar-track, .note-content::-webkit-scrollbar-track, .edit-textarea::-webkit-scrollbar-track {
  background: #282828;
}

.notes-list::-webkit-scrollbar-thumb, .note-content::-webkit-scrollbar-thumb, .edit-textarea::-webkit-scrollbar-thumb {
  background: #504945;
  border-radius: 4px;
}

.notes-list::-webkit-scrollbar-thumb:hover, .note-content::-webkit-scrollbar-thumb:hover, .edit-textarea::-webkit-scrollbar-thumb:hover {
  background: #665c54;
}
</style>
