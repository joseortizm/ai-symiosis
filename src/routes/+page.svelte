<script>
import { invoke } from "@tauri-apps/api/core";
import { onMount } from "svelte";
import { EditorView, basicSetup } from "codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { javascript } from "@codemirror/lang-javascript";
import { python } from "@codemirror/lang-python";
import { rust } from "@codemirror/lang-rust";
import { html } from "@codemirror/lang-html";
import { css } from "@codemirror/lang-css";
import { json } from "@codemirror/lang-json";
import { xml } from "@codemirror/lang-xml";
import { sql } from "@codemirror/lang-sql";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { syntaxHighlighting, HighlightStyle } from "@codemirror/language";

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
let editContainer = $state();
let editorView = null;

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
      
      // Create CodeMirror editor after DOM update
      requestAnimationFrame(() => {
        createCodeMirrorEditor();
      });
    } catch (e) {
      console.error("Failed to load raw note content:", e);
    }
  }
}

function createCodeMirrorEditor() {
  if (!editContainer) return;
  
  // Destroy existing editor if it exists
  if (editorView) {
    editorView.destroy();
    editorView = null;
  }

  // Gruvbox Dark theme
  const gruvboxTheme = EditorView.theme({
    "&": {
      color: "#fbf1c7",
      backgroundColor: "#282828",
      height: "100%"
    },
    ".cm-content": {
      padding: "16px",
      minHeight: "100%",
      caretColor: "#fbf1c7",
      fontFamily: "'JetBrains Mono', 'Consolas', monospace",
      fontSize: "14px",
      lineHeight: "1.5"
    },
    ".cm-focused": {
      outline: "none"
    },
    ".cm-editor": {
      height: "100%"
    },
    ".cm-scroller": {
      fontFamily: "'JetBrains Mono', 'Consolas', monospace",
      height: "100%"
    },
    ".cm-cursor": {
      borderColor: "#fbf1c7"
    },
    ".cm-selectionBackground": {
      backgroundColor: "#504945 !important"
    },
    ".cm-focused .cm-selectionBackground": {
      backgroundColor: "#504945 !important"
    },
    ".cm-activeLine": {
      backgroundColor: "#32302f"
    },
    ".cm-activeLineGutter": {
      backgroundColor: "#32302f"
    },
    ".cm-gutters": {
      backgroundColor: "#32302f",
      color: "#a89984",
      border: "none"
    },
    ".cm-lineNumbers": {
      color: "#a89984"
    },
    ".cm-lineNumbers .cm-gutterElement": {
      padding: "0 8px 0 8px"
    },
    ".cm-foldPlaceholder": {
      backgroundColor: "#504945",
      border: "none",
      color: "#fbf1c7"
    },
    ".cm-searchMatch": {
      backgroundColor: "#fabd2f",
      color: "#282828"
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "#fe8019",
      color: "#282828"
    }
  });

  // Gruvbox syntax highlighting
  const gruvboxHighlighting = syntaxHighlighting(HighlightStyle.define([
    // Comments
    { tag: "comment", color: "#928374", fontStyle: "italic" },
    
    // Keywords
    { tag: "keyword", color: "#fb4934" },
    { tag: "controlKeyword", color: "#fb4934" },
    { tag: "operatorKeyword", color: "#fb4934" },
    
    // Strings
    { tag: "string", color: "#b8bb26" },
    { tag: "special(string)", color: "#fabd2f" },
    
    // Numbers
    { tag: "number", color: "#d3869b" },
    { tag: "integer", color: "#d3869b" },
    { tag: "float", color: "#d3869b" },
    
    // Functions
    { tag: "function(variableName)", color: "#8ec07c" },
    { tag: "function(propertyName)", color: "#8ec07c" },
    
    // Variables
    { tag: "variableName", color: "#fbf1c7" },
    { tag: "propertyName", color: "#83a598" },
    
    // Types
    { tag: "typeName", color: "#fabd2f" },
    { tag: "className", color: "#fabd2f" },
    
    // Operators
    { tag: "operator", color: "#fe8019" },
    { tag: "punctuation", color: "#fbf1c7" },
    
    // Markdown specific
    { tag: "heading1", color: "#fb4934", fontWeight: "bold", fontSize: "1.6em" },
    { tag: "heading2", color: "#fabd2f", fontWeight: "bold", fontSize: "1.4em" },
    { tag: "heading3", color: "#b8bb26", fontWeight: "bold", fontSize: "1.2em" },
    { tag: "heading4", color: "#83a598", fontWeight: "bold", fontSize: "1.1em" },
    { tag: "heading5", color: "#d3869b", fontWeight: "bold" },
    { tag: "heading6", color: "#8ec07c", fontWeight: "bold" },
    
    { tag: "strong", color: "#fe8019", fontWeight: "bold" },
    { tag: "emphasis", color: "#d3869b", fontStyle: "italic" },
    { tag: "strikethrough", textDecoration: "line-through", color: "#928374" },
    { tag: "link", color: "#83a598", textDecoration: "underline" },
    { tag: "monospace", color: "#d3869b", backgroundColor: "#3c3836", padding: "2px 4px", borderRadius: "3px" },
    { tag: "url", color: "#8ec07c" },
    { tag: "quote", color: "#a89984", fontStyle: "italic" },
    { tag: "list", color: "#fe8019" },
    
    // Code blocks
    { tag: "meta", color: "#928374" },
    { tag: "invalid", color: "#fb4934", backgroundColor: "#cc241d" }
  ]));

  // Determine language based on file extension
  function getLanguageExtension(filename) {
    const ext = filename.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'js':
      case 'jsx':
      case 'ts':
      case 'tsx':
        return javascript();
      case 'py':
        return python();
      case 'rs':
        return rust();
      case 'html':
      case 'htm':
        return html();
      case 'css':
        return css();
      case 'json':
        return json();
      case 'xml':
        return xml();
      case 'sql':
        return sql();
      case 'md':
      case 'markdown':
      default:
        return markdown();
    }
  }

  // Custom key bindings
  const customKeymap = keymap.of([
    indentWithTab,
    {
      key: "Ctrl-s",
      run: () => {
        saveNote();
        return true;
      }
    },
    {
      key: "Escape",
      run: () => {
        exitEditMode();
        searchElement?.focus();
        return true;
      }
    }
  ]);

  editorView = new EditorView({
    doc: editContent,
    extensions: [
      basicSetup,
      getLanguageExtension(selectedNote || ''),
      gruvboxTheme,
      gruvboxHighlighting,
      customKeymap,
      EditorView.lineWrapping,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          editContent = update.state.doc.toString();
        }
      })
    ],
    parent: editContainer
  });

  // Focus the editor
  editorView.focus();
}

function exitEditMode() {
  if (editorView) {
    editorView.destroy();
    editorView = null;
  }
  isEditMode = false;
}

async function saveNote() {
  if (!selectedNote || !editContent) return;
  
  try {
    // Get current content from editor if it exists
    const contentToSave = editorView ? editorView.state.doc.toString() : editContent;
    
    await invoke("save_note", { 
      noteName: selectedNote, 
      content: contentToSave 
    });
    
    // Refresh the note content after saving
    const content = await invoke("get_note_content", { noteName: selectedNote });
    noteContent = content;
    highlightedContent = highlightSearchTerms(content, lastQuery);
    
    isEditMode = false;
    if (editorView) {
      editorView.destroy();
      editorView = null;
    }
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
    if (editorView) editorView.destroy();
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
          <div bind:this={editContainer} class="editor-container"></div>
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

.editor-container {
  flex: 1;
  height: 100%;
  background-color: #282828;
  border: 2px solid transparent;
  transition: border-color 0.2s ease;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.editor-container:focus-within {
  border-color: #83a598;
}

/* Ensure CodeMirror takes full height */
.editor-container :global(.cm-editor) {
  height: 100% !important;
}

.editor-container :global(.cm-scroller) {
  height: 100% !important;
  overflow-y: auto !important;
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
.notes-list::-webkit-scrollbar, .note-content::-webkit-scrollbar, .editor-container::-webkit-scrollbar {
  width: 8px;
}

.notes-list::-webkit-scrollbar-track, .note-content::-webkit-scrollbar-track, .editor-container::-webkit-scrollbar-track {
  background: #282828;
}

.notes-list::-webkit-scrollbar-thumb, .note-content::-webkit-scrollbar-thumb, .editor-container::-webkit-scrollbar-thumb {
  background: #504945;
  border-radius: 4px;
}

.notes-list::-webkit-scrollbar-thumb:hover, .note-content::-webkit-scrollbar-thumb:hover, .editor-container::-webkit-scrollbar-thumb:hover {
  background: #665c54;
}
</style>
