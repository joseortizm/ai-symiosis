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
  let showDeleteDialog = $state(false);
  let showCreateDialog = $state(false);
  let newNoteName = $state('');
  let highlightsCleared = $state(false);
  let createDialogInput = $state();
  let deleteKeyCount = $state(0);
  let deleteKeyTimeout = $state();
  let deletionDialog;

  let searchAbortController = null;
  let contentAbortController = null;

  function processContentForDisplay(content, query) {
    if (!query.trim() || highlightsCleared) {
      return content;
    }
    const escapedQuery = query.replace(/[.*+?^${}()|[\\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escapedQuery})`, 'gi');
    return content.replace(regex, '<mark class="highlight">$1</mark>');
  }

  function scrollToFirstMatch() {
    if (noteContentElement && query.trim() && !highlightsCleared) {
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
    // Reset highlights cleared flag when search changes
    if (newQuery.trim()) {
      highlightsCleared = false;
    }
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

  async function getNoteContent(noteName) {
    const content = await invoke("get_note_content", { noteName });
    return content;
  }

  async function deleteNote() {
    if (!selectedNote) return;

    try {
      await invoke("delete_note", { noteName: selectedNote });
      // Refresh the notes list
      await loadNotesImmediate(searchInput);
      showDeleteDialog = false;
      deleteKeyCount = 0;
      clearTimeout(deleteKeyTimeout);
      // Return focus to search
      await tick();
      searchElement?.focus();
    } catch (e) {
      console.error("Failed to delete note:", e);
      alert(`Failed to delete note: ${e}`);
    }
  }

  async function createNote() {
    if (!newNoteName.trim()) return;

    let noteName = newNoteName.trim();
    // Auto-add .md extension if no extension provided
    if (!noteName.includes('.')) {
      noteName += '.md';
    }

    try {
      await invoke("create_new_note", { noteName });
      // Refresh the notes list
      await loadNotesImmediate(searchInput);
      // Select the new note
      const noteIndex = filteredNotes.findIndex(note => note === noteName);
      if (noteIndex >= 0) {
        selectedIndex = noteIndex;
      }
      showCreateDialog = false;
      newNoteName = '';
      // Return focus to search
      await tick();
      searchElement?.focus();
    } catch (e) {
      console.error("Failed to create note:", e);
      alert(`Failed to create note: ${e}`);
    }
  }

  function openCreateDialog() {
    // Pre-fill with search query if no results and query exists
    if (filteredNotes.length === 0 && query.trim()) {
      newNoteName = query.trim();
    } else {
      newNoteName = '';
    }
    showCreateDialog = true;
  }

  function closeCreateDialog() {
    showCreateDialog = false;
    newNoteName = '';
    searchElement?.focus();
  }

  function openDeleteDialog() {
    showDeleteDialog = true;
    deleteKeyCount = 0;
    clearTimeout(deleteKeyTimeout);
  }

  function closeDeleteDialog() {
    showDeleteDialog = false;
    deleteKeyCount = 0;
    clearTimeout(deleteKeyTimeout);
    searchElement?.focus();
  }

  function handleDeleteKeyPress() {
    deleteKeyCount++;
    if (deleteKeyCount === 1) {
      // Start timeout for first 'D' press
      deleteKeyTimeout = setTimeout(() => {
        deleteKeyCount = 0;
      }, 2000); // Reset after 2 seconds
    } else if (deleteKeyCount === 2) {
      // Second 'D' press - confirm deletion
      clearTimeout(deleteKeyTimeout);
      deleteNote();
    }
  }

  function clearHighlights() {
    highlightsCleared = true;
    highlightedContent = processContentForDisplay(noteContent, query);
  }

  function clearSearch() {
    searchInput = '';
    highlightsCleared = false;
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
    // Clear content when no note is selected
    if (!selectedNote) {
      noteContent = '';
      highlightedContent = '';
      return;
    }

    // Cancel any previous content loading request
    if (contentAbortController) {
      contentAbortController.abort();
    }
    contentAbortController = new AbortController();
    const currentController = contentAbortController;

    try {
      // Load the note content from backend
      const content = await getNoteContent(selectedNote);

      // Only update if request wasn't cancelled
      if (!currentController.signal.aborted) {
        noteContent = content;
        highlightedContent = processContentForDisplay(content, query);

        // Scroll to first search match after DOM updates
        requestAnimationFrame(() => {
          scrollToFirstMatch();
        });
      }
    } catch (e) {
      // Handle errors only if request wasn't cancelled
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

  // Effect to focus and select text in create dialog
  $effect(() => {
    if (showCreateDialog && createDialogInput) {
      tick().then(() => {
        createDialogInput.focus();
        // If text was pre-filled from search query, select all
        if (newNoteName.trim()) {
          createDialogInput.select();
        }
      });
    }
  });

  $effect(async () => {
    if (showDeleteDialog && deletionDialog) {
      await tick();
      deletionDialog.focus();
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
        const rawContent = await getNoteContent(selectedNote);
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
      const content = await getNoteContent(selectedNote);
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
      query,
      highlightsCleared,
    }),
    {
      setSelectedIndex: (value) => selectedIndex = value,
      enterEditMode,
      exitEditMode,
      saveNote,
      invoke,
      showDeleteDialog: () => openDeleteDialog(),
      showCreateDialog: () => openCreateDialog(),
      clearHighlights,
      clearSearch,
    }
  );

  onMount(async () => {
    await tick(); // Ensure DOM is updated and searchElement is bound
    searchElement.focus();
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

  <!-- Delete Confirmation Dialog -->
  {#if showDeleteDialog}
    <div class="dialog-overlay" onclick={closeDeleteDialog}>
      <div
        class="dialog"
        bind:this={deletionDialog}
        tabindex="0"
        onclick={(e) => e.stopPropagation()}
           onkeydown={(e) => {
             if (e.key === 'Escape') {
               e.preventDefault();
               closeDeleteDialog();
             } else if (e.key === 'D' || e.key === 'd') {
               e.preventDefault();
               handleDeleteKeyPress();
             }
           }}>
        <h3>Delete Note</h3>
        <p>Are you sure you want to delete "{selectedNote}"?</p>
        <p class="warning">This action cannot be undone.</p>
        <div class="keyboard-hint">
          <p>Press <kbd>DD</kbd> to confirm or <kbd>Esc</kbd> to cancel</p>
          {#if deleteKeyCount === 1}
            <p class="delete-progress">Press <kbd>D</kbd> again to confirm deletion</p>
          {/if}
        </div>
        <div class="dialog-buttons">
          <button class="btn-cancel" onclick={closeDeleteDialog}>Cancel</button>
          <button class="btn-delete" onclick={deleteNote}>Delete</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Create Note Dialog -->
  {#if showCreateDialog}
    <div class="dialog-overlay" onclick={closeCreateDialog}>
      <div class="dialog" onclick={(e) => e.stopPropagation()}>
        <h3>Create New Note</h3>
        <input
          bind:this={createDialogInput}
          bind:value={newNoteName}
          placeholder="Enter note name (extension will be .md)"
          class="note-name-input"
          onkeydown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              createNote();
            } else if (e.key === 'Escape') {
              e.preventDefault();
              closeCreateDialog();
            }
          }}
        />
        <div class="dialog-buttons">
          <button class="btn-cancel" onclick={closeCreateDialog}>Cancel</button>
          <button class="btn-create" onclick={createNote} disabled={!newNoteName.trim()}>Create</button>
        </div>
      </div>
    </div>
  {/if}
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

  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background-color: #3c3836;
    border: 1px solid #504945;
    border-radius: 8px;
    padding: 24px;
    min-width: 400px;
    max-width: 500px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .dialog h3 {
    margin: 0 0 16px 0;
    color: #ebdbb2;
    font-size: 18px;
    font-weight: 600;
  }

  .dialog p {
    margin: 8px 0;
    color: #d5c4a1;
    line-height: 1.5;
  }

  .warning {
    color: #fb4934 !important;
    font-size: 14px;
    font-style: italic;
  }

  .note-name-input {
    width: 100%;
    padding: 12px;
    margin: 16px 0;
    background-color: #282828;
    border: 1px solid #504945;
    border-radius: 6px;
    color: #ebdbb2;
    font-size: 14px;
    font-family: inherit;
    box-sizing: border-box;
    transition: border-color 0.2s ease, box-shadow 0.2s ease;
  }

  .note-name-input:focus {
    outline: none;
    border-color: #83a598;
    box-shadow: 0 0 0 2px rgba(131, 165, 152, 0.2);
  }

  .keyboard-hint {
    margin: 16px 0;
    padding: 12px;
    background-color: #32302f;
    border-radius: 4px;
    border-left: 3px solid #83a598;
  }

  .keyboard-hint p {
    margin: 4px 0;
    font-size: 13px;
    color: #a89984;
  }

  .delete-progress {
    color: #fe8019 !important;
    font-weight: 500;
  }

  kbd {
    background-color: #504945;
    color: #ebdbb2;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 12px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    border: 1px solid #665c54;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .dialog-buttons {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 24px;
  }

  .dialog-buttons button {
    padding: 8px 16px;
    border-radius: 4px;
    border: none;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-cancel {
    background-color: #504945;
    color: #d5c4a1;
  }

  .btn-cancel:hover {
    background-color: #665c54;
  }

  .btn-delete {
    background-color: #fb4934;
    color: #fbf1c7;
  }

  .btn-delete:hover {
    background-color: #cc241d;
  }

  .btn-create {
    background-color: #b8bb26;
    color: #282828;
  }

  .btn-create:hover:not(:disabled) {
    background-color: #98971a;
  }

  .btn-create:disabled {
    background-color: #504945;
    color: #7c6f64;
    cursor: not-allowed;
  }
</style>
