<script>
import { invoke } from "@tauri-apps/api/core";
import { onMount, tick } from "svelte"; // <-- Import 'tick'

// Test CodeMirror imports step by step
let EditorView, basicSetup;
let markdown, javascript, python, rust, html, css, json, xml, sql;
let keymap, indentWithTab;
let syntaxHighlighting, HighlightStyle, tags;

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
let editContainer = $state(); // This will hold the DOM element reference
let editorView = null;

// Performance optimizations
let searchAbortController = null;
let contentAbortController = null;

// --- ADDED fallback editor creation ---
function createFallbackEditor() {
  if (!editContainer) return;
  editContainer.innerHTML = '<textarea style="width:100%; height:100%; background:#282828; color:#fbf1c7; font-family:\'JetBrains Mono\', monospace; padding:16px; border:none; resize:none;"></textarea>';
  const textarea = editContainer.querySelector('textarea');
  if (textarea) {
    textarea.value = editContent || '';
    textarea.addEventListener('input', () => {
      editContent = textarea.value;
    });
    setTimeout(() => textarea.focus(), 10);
  }
}
// --- END ADDED fallback ---

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

// --- UPDATED enterEditMode using 'tick' ---
async function enterEditMode() {
  if (selectedNote) {
    try {
      // Get raw content for editing (not rendered HTML)
      const rawContent = await invoke("get_note_raw_content", { noteName: selectedNote });
      isEditMode = true;
      editContent = rawContent;

      // --- REPLACED requestAnimationFrame with tick ---
      // Wait for Svelte to update the DOM with the edit container
      await tick();
      // Now create the editor, ensuring editContainer should be available
      createCodeMirrorEditor();

    } catch (e) {
      console.error("Failed to load raw note content:", e);
      // Optionally handle error, maybe revert isEditMode
      // isEditMode = false; // Consider adding this if you want to exit edit mode on error
    }
  }
}
// --- END UPDATED enterEditMode ---

// --- REPLACED createCodeMirrorEditor with the enhanced version ---
function createCodeMirrorEditor() {
  if (!editContainer) {
    console.error('Edit container not found');
    return;
  }
  // Destroy existing editor if it exists
  if (editorView) {
    editorView.destroy();
    editorView = null;
  }
  // Clear container
  editContainer.innerHTML = '';
  // Check if CodeMirror modules are available
  if (typeof EditorView === 'undefined' || !EditorView) {
    console.warn('CodeMirror not available, using fallback textarea');
    createFallbackEditor();
    return;
  }
  try {
    console.log('Creating CodeMirror editor for:', selectedNote);
    // Gruvbox Dark theme
    const gruvboxTheme = EditorView.theme({
      "&": {
        color: "#fbf1c7",
        backgroundColor: "#282828",
        height: "100%",
        fontSize: "14px"
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
      }
    });
    // Gruvbox syntax highlighting
    const gruvboxHighlighting = syntaxHighlighting ? syntaxHighlighting(HighlightStyle.define([
      // Comments
      { tag: tags.comment, color: "#928374", fontStyle: "italic" },
      { tag: tags.lineComment, color: "#928374", fontStyle: "italic" },
      { tag: tags.blockComment, color: "#928374", fontStyle: "italic" },
      // Keywords
      { tag: tags.keyword, color: "#fb4934" },
      { tag: tags.controlKeyword, color: "#fb4934" },
      { tag: tags.operatorKeyword, color: "#fb4934" },
      { tag: tags.modifier, color: "#fb4934" },
      { tag: tags.null, color: "#fb4934" },
      { tag: tags.bool, color: "#fb4934" },
      // Strings
      { tag: tags.string, color: "#b8bb26" },
      { tag: tags.character, color: "#b8bb26" },
      { tag: tags.regexp, color: "#b8bb26" },
      // Numbers
      { tag: tags.number, color: "#d3869b" },
      { tag: tags.integer, color: "#d3869b" },
      { tag: tags.float, color: "#d3869b" },
      // Functions
      { tag: tags.function(tags.variableName), color: "#8ec07c" },
      { tag: tags.function(tags.propertyName), color: "#8ec07c" },
      // Variables
      { tag: tags.variableName, color: "#fbf1c7" },
      { tag: tags.propertyName, color: "#83a598" },
      // Types
      { tag: tags.typeName, color: "#fabd2f" },
      { tag: tags.className, color: "#fabd2f" },
      // Operators
      { tag: tags.operator, color: "#fe8019" },
      { tag: tags.punctuation, color: "#fbf1c7" },
      // Markdown specific
      { tag: tags.heading1, color: "#fb4934", fontWeight: "bold", fontSize: "1.6em" },
      { tag: tags.heading2, color: "#fabd2f", fontWeight: "bold", fontSize: "1.4em" },
      { tag: tags.heading3, color: "#b8bb26", fontWeight: "bold", fontSize: "1.2em" },
      { tag: tags.strong, color: "#fe8019", fontWeight: "bold" },
      { tag: tags.emphasis, color: "#d3869b", fontStyle: "italic" },
      { tag: tags.link, color: "#83a598", textDecoration: "underline" },
      { tag: tags.monospace, color: "#d3869b", backgroundColor: "#3c3836" }
    ])) : null;
    // Determine language based on file extension
    function getLanguageExtension(filename) {
      if (!filename) return markdown ? markdown() : null;
      const ext = filename.split('.').pop()?.toLowerCase();
      console.log('File extension:', ext);
      switch (ext) {
        case 'js':
        case 'jsx':
        case 'ts':
        case 'tsx':
          return javascript ? javascript() : null;
        case 'py':
          return python ? python() : null;
        case 'rs':
          return rust ? rust() : null;
        case 'html':
        case 'htm':
          return html ? html() : null;
        case 'css':
          return css ? css() : null;
        case 'json':
          return json ? json() : null;
        case 'xml':
          return xml ? xml() : null;
        case 'sql':
          return sql ? sql() : null;
        case 'md':
        case 'markdown':
        default:
          return markdown ? markdown() : null;
      }
    }
    // Custom key bindings
    const customKeymap = keymap ? keymap.of([
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
    ]) : [];
    // Build extensions array
    const extensions = [
      basicSetup,
      getLanguageExtension(selectedNote),
      gruvboxTheme,
      gruvboxHighlighting,
      customKeymap,
      EditorView.lineWrapping,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          editContent = update.state.doc.toString();
        }
      })
    ].filter(Boolean); // Remove null/undefined extensions
    console.log('Extensions loaded:', extensions.length);
    // Create the editor
    editorView = new EditorView({
      doc: editContent || '',
      extensions,
      parent: editContainer
    });
    console.log('CodeMirror editor created successfully');
    // Focus the editor
    setTimeout(() => {
      if (editorView) {
        editorView.focus();
      }
    }, 100);
  } catch (error) {
    console.error('Failed to create CodeMirror editor:', error);
    console.log('Falling back to textarea');
    createFallbackEditor();
  }
}
// --- END REPLACED createCodeMirrorEditor ---

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

// --- REPLACED onMount with dynamic import loading ---
// Import CodeMirror modules with error handling
async function loadCodeMirrorModules() {
  try {
    // Core CodeMirror
    const coreModule = await import("codemirror");
    EditorView = coreModule.EditorView;
    basicSetup = coreModule.basicSetup;
    console.log('✅ Core CodeMirror loaded');
    // Languages
    try {
      const mdModule = await import("@codemirror/lang-markdown");
      markdown = mdModule.markdown;
      console.log('✅ Markdown loaded');
    } catch (e) { console.warn('❌ Markdown not loaded:', e.message); }
    try {
      const jsModule = await import("@codemirror/lang-javascript");
      javascript = jsModule.javascript;
      console.log('✅ JavaScript loaded');
    } catch (e) { console.warn('❌ JavaScript not loaded:', e.message); }
    try {
      const pyModule = await import("@codemirror/lang-python");
      python = pyModule.python;
      console.log('✅ Python loaded');
    } catch (e) { console.warn('❌ Python not loaded:', e.message); }
    try {
      const rustModule = await import("@codemirror/lang-rust");
      rust = rustModule.rust;
      console.log('✅ Rust loaded');
    } catch (e) { console.warn('❌ Rust not loaded:', e.message); }
    try {
      const htmlModule = await import("@codemirror/lang-html");
      html = htmlModule.html;
      console.log('✅ HTML loaded');
    } catch (e) { console.warn('❌ HTML not loaded:', e.message); }
    try {
      const cssModule = await import("@codemirror/lang-css");
      css = cssModule.css;
      console.log('✅ CSS loaded');
    } catch (e) { console.warn('❌ CSS not loaded:', e.message); }
    try {
      const jsonModule = await import("@codemirror/lang-json");
      json = jsonModule.json;
      console.log('✅ JSON loaded');
    } catch (e) { console.warn('❌ JSON not loaded:', e.message); }
    // Commands and keymap
    try {
      const commandsModule = await import("@codemirror/commands");
      indentWithTab = commandsModule.indentWithTab;
      console.log('✅ Commands loaded');
    } catch (e) { console.warn('❌ Commands not loaded:', e.message); }
    try {
      const viewModule = await import("@codemirror/view");
      keymap = viewModule.keymap;
      console.log('✅ View/keymap loaded');
    } catch (e) { console.warn('❌ View/keymap not loaded:', e.message); }
    // Syntax highlighting
    try {
      const langModule = await import("@codemirror/language");
      syntaxHighlighting = langModule.syntaxHighlighting;
      HighlightStyle = langModule.HighlightStyle;
      console.log('✅ Language/highlighting loaded');
    } catch (e) { console.warn('❌ Language/highlighting not loaded:', e.message); }
    try {
      const highlightModule = await import("@lezer/highlight");
      tags = highlightModule.tags;
      console.log('✅ Highlight tags loaded');
    } catch (e) { console.warn('❌ Highlight tags not loaded:', e.message); }
    console.log('🎉 CodeMirror modules loading complete');
    return true;
  } catch (error) {
    console.error('❌ Failed to load CodeMirror core:', error);
    return false;
  }
}

// Update your onMount to load CodeMirror
onMount(async () => {
  searchElement.focus();
  // Load CodeMirror modules
  const codeMirrorLoaded = await loadCodeMirrorModules();
  if (codeMirrorLoaded) {
    console.log('CodeMirror ready for use');
  } else {
    console.log('CodeMirror not available, will use fallback editor');
  }
  // Load initial notes
  loadNotesImmediate('');
});
// --- END REPLACED onMount ---

// Clean up on component destroy
onMount(() => { // Note: This second onMount is okay, it adds another cleanup function
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
          <!-- This div is bound to 'editContainer' -->
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
              // invoke("open_external_link", { url: target.href }); // Ensure this command exists
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
