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
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\// Function to scroll selected item into view');
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

function handleKeydown(event) {
  if (isSearchInputFocused) {
    switch (event.key) {
      case 'Enter':
        event.preventDefault();
        if (filteredNotes.length > 0 && selectedNote) {
          // Focus on note content instead of opening
          noteContentElement?.focus();
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

  // Handle note content navigation
  if (isNoteContentFocused) {
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
    }
  }

  if (filteredNotes.length === 0) return;

  // Global navigation when not in search or note content
  if (!isSearchInputFocused && !isNoteContentFocused) {
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
          noteContentElement?.focus();
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
    placeholder="Search notes..."
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
      <div class="note-header">
        <h3>{selectedNote}</h3>
      </div>
      <div class="note-content"
           bind:this={noteContentElement}
           tabindex="0"
           onfocus={() => isNoteContentFocused = true}
           onblur={() => isNoteContentFocused = false}>
        {#if lastQuery.trim()}
          <div class="note-text">{@html highlightedContent}</div>
        {:else}
          <pre class="note-text-plain">{noteContent}</pre>
        {/if}
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
  /* Enable hardware acceleration */
  transform: translateZ(0);
  will-change: scroll-position;
  /* Smooth scrolling */
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
  /* Enable hardware acceleration */
  transform: translateZ(0);
  will-change: scroll-position;
  /* Make it focusable */
  outline: none;
  border: 2px solid transparent;
  transition: border-color 0.2s ease;
}

.note-content:focus {
  border-color: #83a598;
}

.note-text {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: 'Inter', sans-serif;
  font-size: 0.9em;
  line-height: 1.6;
  color: #ebdbb2;
}

.note-text-plain {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: 'Inter', sans-serif;
  font-size: 0.9em;
  line-height: 1.6;
  color: #ebdbb2;
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
  align-items: center;
  justify-content: center;
  color: #928374;
  font-style: italic;
}

ul {
  list-style: none;
  padding: 0;
  margin: 0;
  /* Optimize for frequent updates */
  contain: content;
}

li {
  margin: 0;
  /* Optimize rendering */
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
  /* Optimize button rendering */
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
.notes-list::-webkit-scrollbar, .note-content::-webkit-scrollbar {
  width: 8px;
}

.notes-list::-webkit-scrollbar-track, .note-content::-webkit-scrollbar-track {
  background: #282828;
}

.notes-list::-webkit-scrollbar-thumb, .note-content::-webkit-scrollbar-thumb {
  background: #504945;
  border-radius: 4px;
}

.notes-list::-webkit-scrollbar-thumb:hover, .note-content::-webkit-scrollbar-thumb:hover {
  background: #665c54;
}
</style>
