<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, tick } from "svelte";
  import SearchInput from "../lib/components/SearchInput.svelte";
  import NoteList from "../lib/components/NoteList.svelte";
  import NoteView from "../lib/components/NoteView.svelte";
  import { createKeyboardHandler } from '../lib/keyboardHandler.js';

  let filteredNotes = $state([]);
  let selectedNote = $state(null);
  let selectedIndex = $state(-1);
  let searchInput = $state('');
  let noteContent = $state('');
  let searchElement = $state();
  let noteListElement = $state();
  let noteContentElement = $state();
  let isSearchInputFocused = $state(false);
  let isNoteContentFocused = $state(false);
  let isLoading = $state(false);
  let query = $state('');
  let highlightedContent = $state('');
  let isEditMode = $state(false);
  let editContent = $state('');

  let searchAbortController = null;
  let contentAbortController = null;

  function processContentForDisplay(content, query) {
    if (!query.trim()) {
      return content;
    }
    const escapedQuery = query.replace(/[.*+?^${}()|[\\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escapedQuery})`, 'gi');
    return content.replace(regex, '<mark class="highlight">$1</mark>');
  }

  function scrollToFirstMatch() {
    if (noteContentElement && query.trim()) {
      setTimeout(() => {
        const firstMatch = noteContentElement.querySelector('.highlight');
        if (firstMatch) {
          firstMatch.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
      }, 100);
    }
  }

  function scrollToSelected() {
    if (noteListElement && selectedIndex >= 0) {
      const selectedButton = noteListElement.children[selectedIndex]?.querySelector('button');
      if (selectedButton) {
        selectedButton.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }
    }
  }

  let searchTimeout;
  function debounceSearch(newQuery) {
    if (newQuery === query) return;
    // Update query immediately for highlighting
    query = newQuery;
    // Cancel previous search request and timer
    clearTimeout(searchTimeout);
    searchAbortController?.abort();
    // Start new timer to search after user stops typing
    searchTimeout = setTimeout(() => {
      loadNotesImmediate(newQuery);
    }, 100);
  }

  async function loadNotesImmediate(searchQuery) {
    if (searchAbortController) {
      searchAbortController.abort();
    }
    searchAbortController = new AbortController();
    const currentController = searchAbortController;
    try {
      isLoading = true;
      const newNotes = await invoke("search_notes", { query: searchQuery });
      if (currentController.signal.aborted) {
        return;
      }
      if (JSON.stringify(newNotes) !== JSON.stringify(filteredNotes)) {
        filteredNotes = newNotes;
        if (newNotes.length === 0) {
          selectedIndex = -1;
        } else {
          selectedIndex = 0;
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

  $effect(() => {
    debounceSearch(searchInput);
  });

  $effect(() => {
    const newSelectedNote = filteredNotes.length > 0 && selectedIndex !== -1
      ? filteredNotes[selectedIndex]
      : null;
    if (newSelectedNote !== selectedNote) {
      selectedNote = newSelectedNote;
      isEditMode = false;
    }
  });

  $effect(() => {
    if (selectedIndex >= 0) {
      requestAnimationFrame(() => {
        scrollToSelected();
      });
    }
  });

  $effect(async () => {
    if (!selectedNote) {
      noteContent = '';
      highlightedContent = '';
      return;
    }
    if (contentAbortController) {
      contentAbortController.abort();
    }
    contentAbortController = new AbortController();
    const currentController = contentAbortController;
    try {
      const content = await invoke("get_note_content", { noteName: selectedNote });
      if (!currentController.signal.aborted) {
        noteContent = content;
        highlightedContent = processContentForDisplay(content, query);
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

  $effect(() => {
    if (noteContent) {
      highlightedContent = processContentForDisplay(noteContent, query);
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
        // You'll need to add this function to your Rust code or modify get_note_content
        // For now, let's try to get the raw content another way
        const rawContent = await invoke("get_note_raw_content", { noteName: selectedNote });
        isEditMode = true;
        editContent = rawContent;
      } catch (e) {
        console.error("Failed to load raw note content:", e);
        // Fallback: try to extract text from HTML content
        const tempDiv = document.createElement('div');
        tempDiv.innerHTML = noteContent;
        editContent = tempDiv.textContent || tempDiv.innerText || '';
        isEditMode = true;
      }
    }
  }

  function exitEditMode() {
    isEditMode = false;
  }

  async function saveNote() {
    if (!selectedNote || !editContent) return;
    try {
      await invoke("save_note", {
        noteName: selectedNote,
        content: editContent
      });
      const content = await invoke("get_note_content", { noteName: selectedNote });
      noteContent = content;
      highlightedContent = processContentForDisplay(content, query);
      isEditMode = false;
    } catch (e) {
      console.error("Failed to save note:", e);
    }
  }

  const handleKeydown = createKeyboardHandler(
    () => ({
      isSearchInputFocused,
      isEditMode,
      isNoteContentFocused,
      selectedIndex,
      filteredNotes,
      selectedNote,
      noteContentElement,
      searchElement,
    }),
    {
      setSelectedIndex: (value) => selectedIndex = value,
      enterEditMode,
      exitEditMode,
      saveNote,
      invoke,
    }
  );

  onMount(async () => {
    await tick(); // Ensure DOM is updated and searchElement is bound
    console.log('onMount: Attempting to focus searchElement', searchElement);
    setTimeout(() => {
      if (searchElement) {
        searchElement.focus();
      }
    }, 0); // Small delay to ensure DOM is ready
    loadNotesImmediate('');
    return () => {
      if (searchAbortController) searchAbortController.abort();
      if (contentAbortController) contentAbortController.abort();
      clearTimeout(searchTimeout);
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />
<main class="container">
  <SearchInput
    bind:value={searchInput}
    onFocus={() => isSearchInputFocused = true}
    onBlur={() => isSearchInputFocused = false}
    bind:element={searchElement}
  />
  <NoteList
    notes={filteredNotes}
    selectedIndex={selectedIndex}
    isLoading={isLoading}
    onSelectNote={selectNote}
    bind:listElement={noteListElement}
  />
  <NoteView
    selectedNote={selectedNote}
    isEditMode={isEditMode}
    bind:editContent={editContent}
    highlightedContent={highlightedContent}
    onSave={saveNote}
    onExitEditMode={exitEditMode}
    onEnterEditMode={enterEditMode}
    bind:noteContentElement={noteContentElement}
    bind:isNoteContentFocused={isNoteContentFocused}
  />
</main>

<style>
  :global(body) {
    margin: 0;
    background-color: #282c34;
  }
  .container {
    margin: 0;
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #282828;
    color: #ebdbb2;
    font-family: 'Inter', sans-serif;
  }
</style>
