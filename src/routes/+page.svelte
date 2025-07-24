<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import SearchInput from "../lib/components/SearchInput.svelte";
  import NoteList from "../lib/components/NoteList.svelte";
  import NoteView from "../lib/components/NoteView.svelte";

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
  let isEditMode = $state(false);
  let editContent = $state('');

  let searchAbortController = null;
  let contentAbortController = null;

  function highlightSearchTerms(content, query) {
    if (!query.trim()) {
      return content;
    }
    const escapedQuery = query.replace(/[.*+?^${}()|[\\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escapedQuery})`, 'gi');
    return content.replace(regex, '<mark class="highlight">$1</mark>');
  }

  function scrollToFirstMatch() {
    if (noteContentElement && lastQuery.trim()) {
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
  function debounceSearch(query) {
    if (query === lastQuery) return;
    clearTimeout(searchTimeout);
    if (searchAbortController) {
      searchAbortController.abort();
    }
    searchTimeout = setTimeout(() => {
      loadNotesImmediate(query);
    }, 100);
  }

  async function loadNotesImmediate(query) {
    if (searchAbortController) {
      searchAbortController.abort();
    }
    searchAbortController = new AbortController();
    const currentController = searchAbortController;
    try {
      isLoading = true;
      lastQuery = query;
      const newNotes = await invoke("list_notes", { query });
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
        highlightedContent = highlightSearchTerms(content, lastQuery);
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
        const rawContent = await invoke("get_note_raw_content", { noteName: selectedNote });
        isEditMode = true;
        editContent = rawContent;
      } catch (e) {
        console.error("Failed to load raw note content:", e);
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
          event.preventDefault();
          selectedIndex = Math.max(0, selectedIndex - 1);
          break;
        case 'ArrowDown':
          event.preventDefault();
          selectedIndex = Math.min(filteredNotes.length - 1, selectedIndex + 1);
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
    if (isNoteContentFocused && !isEditMode) {
      switch (event.key) {
        case 'ArrowUp':
          event.preventDefault();
          noteContentElement.scrollBy({ top: -50, behavior: 'smooth' });
          return;
        case 'ArrowDown':
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
    if (!isSearchInputFocused && !isNoteContentFocused && !isEditMode) {
      switch (event.key) {
        case 'ArrowUp':
          event.preventDefault();
          selectedIndex = Math.max(0, selectedIndex - 1);
          break;
        case 'ArrowDown':
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

  onMount(() => {
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
