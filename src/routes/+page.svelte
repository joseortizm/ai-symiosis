<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMount, tick } from "svelte";
import { listen } from "@tauri-apps/api/event";
import SearchInput from "../lib/components/SearchInput.svelte";
import NoteList from "../lib/components/NoteList.svelte";
import NoteView from "../lib/components/NoteView.svelte";
import Editor from "../lib/components/Editor.svelte";
import ConfirmationDialog from "../lib/components/ConfirmationDialog.svelte";
import { createKeyboardHandler } from '../lib/keyboardHandler';
import { setAppContext } from '../lib/context/app.svelte';
import { contentHighlighter } from '../lib/utils/contentHighlighting.svelte';
import { searchManager } from '../lib/utils/searchManager.svelte';

// Tauri API Response Types
interface SearchNotesResponse {
  [key: string]: string[];
}

type TauriInvokeResponse<T> = Promise<T>;

// Create reactive state with $state rune
const appState = $state({
  // Search state
  searchInput: '',
  query: '',
  isLoading: false,
  areHighlightsCleared: false,

  // Selection state
  filteredNotes: [] as string[],
  selectedNote: null as string | null,
  selectedIndex: -1,

  // Editor state
  noteContent: '',
  highlightedContent: '',
  isEditMode: false,
  editContent: '',
  isEditorDirty: false,
  nearestHeaderText: '',

  // Dialog state
  showDeleteDialog: false,
  showCreateDialog: false,
  showRenameDialog: false,
  showConfigDialog: false,
  showUnsavedChangesDialog: false,
  newNoteName: '',
  newNoteNameForRename: '',
  configContent: '',
  deleteKeyPressCount: 0,
  deleteKeyResetTimeout: undefined as number | undefined,

  // UI state
  isSearchInputFocused: false,
  isNoteContentFocused: false,
  searchElement: null as HTMLInputElement | null,
  noteListElement: null as HTMLElement | null,
  noteContentElement: null as HTMLElement | null,
  renameDialogInputElement: null as HTMLInputElement | null,
  createDialogInputElement: null as HTMLInputElement | null,
});

// svelte-ignore non_reactive_update
let deleteDialogElement: HTMLElement;

let contentRequestController: AbortController | null = null;



function scrollToFirstMatch(): void {
  contentHighlighter.scrollToFirstMatch();
}

function scrollToSelected(): void {
  if (appState.noteListElement && appState.selectedIndex >= 0) {
    const selectedButton = appState.noteListElement.children[appState.selectedIndex]?.querySelector('button');
    if (selectedButton) {
      selectedButton.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
  }
}


async function getNoteContent(noteName: string): Promise<string> {
  const content = await invoke<string>("get_note_content", { noteName });
  return content;
}

async function deleteNote(): Promise<void> {
  if (!appState.selectedNote) return;

  try {
    await invoke<void>("delete_note", { noteName: appState.selectedNote });
    // Refresh the notes list
    const notes = await searchManager.searchImmediate(appState.searchInput);
    appState.filteredNotes = notes;
    appState.showDeleteDialog = false;
    appState.deleteKeyPressCount = 0;
    clearTimeout(appState.deleteKeyResetTimeout);
    // Return focus to search
    await tick();
    appState.searchElement?.focus();
  } catch (e) {
    console.error("Failed to delete note:", e);
    alert(`Failed to delete note: ${e}`);
  }
}

async function createNote(): Promise<void> {
  if (!appState.newNoteName.trim()) return;

  let noteName = appState.newNoteName.trim();
  // Auto-add .md extension if no extension provided
  if (!noteName.includes('.')) {
    noteName += '.md';
  }

  try {
    await invoke<void>("create_new_note", { noteName });
    // Refresh the notes list
    const notes = await searchManager.searchImmediate(appState.searchInput);
    appState.filteredNotes = notes;
    // Select the new note
    const noteIndex = appState.filteredNotes.findIndex(note => note === noteName);
    if (noteIndex >= 0) {
      appState.selectedIndex = noteIndex;
    }
    appState.showCreateDialog = false;
    appState.newNoteName = '';
    // Return focus to search
    await tick();
    appState.searchElement?.focus();
  } catch (e) {
    console.error("Failed to create note:", e);
    alert(`Failed to create note: ${e}`);
  }
}

async function renameNote(): Promise<void> {
  if (!appState.newNoteNameForRename.trim() || !appState.selectedNote) return;

  let newName = appState.newNoteNameForRename.trim();
  if (!newName.includes('.')) {
    newName += '.md';
  }

  try {
    await invoke<void>("rename_note", { oldName: appState.selectedNote, newName: newName });
    const notes = await searchManager.searchImmediate(appState.searchInput);
    appState.filteredNotes = notes;
    const noteIndex = appState.filteredNotes.findIndex(note => note === newName);
    if (noteIndex >= 0) {
      appState.selectedIndex = noteIndex;
    }
    closeRenameDialog();
  } catch (e) {
    console.error("Failed to rename note:", e);
    alert(`Failed to rename note: ${e}`);
  }
}

function openRenameDialog(): void {
  if (appState.selectedNote) {
    appState.newNoteNameForRename = appState.selectedNote.endsWith('.md') ? appState.selectedNote.slice(0, -3) : appState.selectedNote;
    appState.showRenameDialog = true;
  }
}

function closeRenameDialog(): void {
  appState.showRenameDialog = false;
  appState.newNoteNameForRename = '';
  appState.searchElement?.focus();
}

function openCreateDialog(): void {
  // Pre-fill with search query if no results and query exists
  if (!appState.highlightedContent.trim() && appState.query.trim()) {
    appState.newNoteName = appState.query.trim();
  } else {
    appState.newNoteName = '';
  }
  appState.showCreateDialog = true;
}

function closeCreateDialog(): void {
  appState.showCreateDialog = false;
  appState.newNoteName = '';
  appState.searchElement?.focus();
}

function openDeleteDialog(): void {
  appState.showDeleteDialog = true;
  appState.deleteKeyPressCount = 0;
  clearTimeout(appState.deleteKeyResetTimeout);
}

function closeDeleteDialog(): void {
  appState.showDeleteDialog = false;
  appState.deleteKeyPressCount = 0;
  clearTimeout(appState.deleteKeyResetTimeout);
  appState.searchElement?.focus();
}

async function openConfigDialog(): Promise<void> {
  try {
    const content = await invoke<string>("get_config_content");
    appState.configContent = content;
    appState.showConfigDialog = true;
  } catch (e) {
    console.error("Failed to load config:", e);
  }
}

function closeConfigDialog(): void {
  appState.showConfigDialog = false;
  appState.configContent = '';
  appState.searchElement?.focus();
}

async function saveConfig(): Promise<void> {
  try {
    await invoke<void>("save_config_content", { content: appState.configContent });
    await invoke<void>("refresh_cache");
    closeConfigDialog();
    appState.searchElement?.focus();
    const notes = await searchManager.searchImmediate('');
    appState.filteredNotes = notes;
  } catch (e) {
    console.error("Failed to save config:", e);
    alert(`Failed to save config: ${e}`);
  }
}

function handleDeleteKeyPress(): void {
  appState.deleteKeyPressCount++;
  if (appState.deleteKeyPressCount === 1) {
    // Start timeout for first 'D' press
    appState.deleteKeyResetTimeout = setTimeout(() => {
      appState.deleteKeyPressCount = 0;
    }, 2000); // Reset after 2 seconds
  } else if (appState.deleteKeyPressCount === 2) {
    // Second 'D' press - confirm deletion
    clearTimeout(appState.deleteKeyResetTimeout);
    deleteNote();
  }
}

function clearHighlights(): void {
  appState.areHighlightsCleared = true;
  appState.highlightedContent = contentHighlighter.highlighted;
}

function clearSearch(): void {
  appState.searchInput = '';
  appState.areHighlightsCleared = false;
}

$effect(() => {
  const newQuery = appState.searchInput;
  if (newQuery === appState.query) return;
  
  appState.query = newQuery;
  if (newQuery.trim()) {
    appState.areHighlightsCleared = false;
  }
  
  searchManager.updateState({ searchInput: newQuery });
});

$effect(() => {
  appState.isLoading = searchManager.isLoading;
});

$effect(() => {
  const notes = searchManager.filteredNotes;
  if (JSON.stringify(notes) !== JSON.stringify(appState.filteredNotes)) {
    appState.filteredNotes = notes;
    if (notes.length === 0) {
      appState.selectedIndex = -1;
    } else {
      appState.selectedIndex = 0;
    }
  }
});

$effect(() => {
  const newSelectedNote = appState.filteredNotes.length > 0 && appState.selectedIndex !== -1
    ? appState.filteredNotes[appState.selectedIndex]
    : null;
  if (newSelectedNote !== appState.selectedNote) {
    appState.selectedNote = newSelectedNote;
    appState.isEditMode = false;
  }
});

$effect(() => {
  if (appState.selectedIndex >= 0) {
    requestAnimationFrame(() => {
      scrollToSelected();
    });
  }
});

$effect(() => {
  // Clear content when no note is selected
  if (!appState.selectedNote) {
    appState.noteContent = '';
    appState.highlightedContent = '';
    return;
  }

  // Cancel any previous content loading request
  if (contentRequestController) {
    contentRequestController.abort();
  }
  contentRequestController = new AbortController();
  const currentController = contentRequestController;

  // Handle async loading
  (async () => {
    try {
      // Load the note content from backend
      const content = await getNoteContent(appState.selectedNote!);

      // Only update if request wasn't cancelled
      if (!currentController.signal.aborted) {
        appState.noteContent = content;
        appState.highlightedContent = contentHighlighter.highlighted;

        // Scroll to first search match after DOM updates
        requestAnimationFrame(() => {
          scrollToFirstMatch();
        });
      }
    } catch (e) {
      // Handle errors only if request wasn't cancelled
      if (!currentController.signal.aborted) {
        console.error("Failed to load note content:", e);
        appState.noteContent = `Error loading note: ${e}`;
        appState.highlightedContent = appState.noteContent;
      }
    }
  })();
});

$effect(() => {
  contentHighlighter.updateState({
    query: appState.query,
    content: appState.noteContent,
    areHighlightsCleared: appState.areHighlightsCleared,
    noteContentElement: appState.noteContentElement
  });
  appState.highlightedContent = contentHighlighter.highlighted;
});

// Effect to focus and select text in create dialog
$effect(() => {
  if (appState.showCreateDialog && appState.createDialogInputElement) {
    tick().then(() => {
      appState.createDialogInputElement!.focus();
      // If text was pre-filled from search query, select all
      if (appState.newNoteName.trim()) {
        appState.createDialogInputElement!.select();
      }
    });
  }
});

$effect(() => {
  if (appState.showDeleteDialog && deleteDialogElement) {
    tick().then(() => {
      deleteDialogElement.focus();
    });
  }
});

$effect(() => {
  if (appState.showRenameDialog && appState.renameDialogInputElement) {
    tick().then(() => {
      appState.renameDialogInputElement!.focus();
      appState.renameDialogInputElement!.select();
    });
  }
});

function selectNote(note: string, index: number): void {
  if (appState.selectedIndex !== index) {
    appState.selectedIndex = index;
  }
}

async function enterEditMode(): Promise<void> {
  if (appState.selectedNote) {
    if (appState.noteContentElement) {
      const rect = appState.noteContentElement.getBoundingClientRect();
      const headers = appState.noteContentElement.querySelectorAll('h1, h2, h3, h4, h5, h6');

      for (const header of headers) {
        const headerRect = header.getBoundingClientRect();
        if (headerRect.top >= rect.top) {
          appState.nearestHeaderText = header.textContent?.trim() || '';
          break;
        }
      }
    }

    try {
      const rawContent = await invoke<string>("get_note_raw_content", { noteName: appState.selectedNote });
      appState.isEditMode = true;
      appState.editContent = rawContent;
    } catch (e) {
      console.error("Failed to load raw note content:", e);
      // Fallback: try to extract text from HTML content
      const tempDiv = document.createElement('div');
      tempDiv.innerHTML = appState.noteContent;
      appState.editContent = tempDiv.textContent || tempDiv.innerText || '';
      appState.isEditMode = true;
    }
  }
}

function exitEditMode(): void {
  appState.isEditMode = false;
  appState.searchElement?.focus();
}

function showExitEditDialog(): void {
  appState.showUnsavedChangesDialog = true;
}

function handleSaveAndExit(): void {
  appState.showUnsavedChangesDialog = false;
  saveNote();
  exitEditMode();
}

function handleDiscardAndExit(): void {
  appState.showUnsavedChangesDialog = false;
  exitEditMode();
}

async function saveNote(): Promise<void> {
  if (!appState.selectedNote || !appState.editContent) return;
  try {
    await invoke<void>("save_note", {
      noteName: appState.selectedNote,
      content: appState.editContent
    });

    // refresh the database to include the new file
    await invoke<void>("refresh_cache");

    // Refresh the notes list to sync with database
    const notes = await searchManager.searchImmediate(appState.searchInput);
    appState.filteredNotes = notes;

    // Reload the current note content
    const content = await getNoteContent(appState.selectedNote);
    appState.noteContent = content;
    appState.highlightedContent = contentHighlighter.highlighted;
    appState.isEditMode = false;

    // Return focus to search after UI updates
    await tick();
    appState.searchElement?.focus();
  } catch (e) {
    console.error("Failed to save note:", e);
  }
}

const handleKeydown = createKeyboardHandler(
  () => ({
    isSearchInputFocused: appState.isSearchInputFocused,
    isEditMode: appState.isEditMode,
    isNoteContentFocused: appState.isNoteContentFocused,
    selectedIndex: appState.selectedIndex,
    filteredNotes: appState.filteredNotes,
    selectedNote: appState.selectedNote,
    noteContentElement: appState.noteContentElement,
    searchElement: appState.searchElement,
    query: appState.query,
    areHighlightsCleared: appState.areHighlightsCleared,
    showConfigDialog: appState.showConfigDialog,
    isEditorDirty: appState.isEditorDirty,
  }),
  {
    setSelectedIndex: (value: number) => appState.selectedIndex = value,
    enterEditMode,
    exitEditMode,
    showExitEditDialog,
    saveNote,
    invoke,
    showDeleteDialog: () => openDeleteDialog(),
    showCreateDialog: () => openCreateDialog(),
    showRenameDialog: () => openRenameDialog(),
    clearHighlights,
    clearSearch,
  }
);

// Set up the context - pass the state object directly with actions
setAppContext({
  // Pass the reactive state object directly (don't spread it)
  state: appState,

  // Action functions
  selectNote,
  deleteNote,
  createNote,
  renameNote,
  saveNote,
  enterEditMode,
  exitEditMode,
  showExitEditDialog,
  handleSaveAndExit,
  handleDiscardAndExit,
  openCreateDialog,
  closeCreateDialog,
  openRenameDialog,
  closeRenameDialog,
  openDeleteDialog,
  closeDeleteDialog,
  openConfigDialog,
  closeConfigDialog,
  saveConfig,
  handleDeleteKeyPress,
  clearHighlights,
  clearSearch,
  invoke,
});

onMount(() => {
  (async () => {
    await tick();

    const configExists = await invoke<boolean>("config_exists");
    if (!configExists) {
      openConfigDialog();
    } else {
      appState.searchElement?.focus();
      const notes = await searchManager.searchImmediate('');
      appState.filteredNotes = notes;
    }

    const unlisten = await listen("open-preferences", () => {
      openConfigDialog();
    });

    return () => {
      searchManager.abort();
      if (contentRequestController) contentRequestController.abort();
      unlisten();
    };
  })();
});
</script>

<svelte:window onkeydown={handleKeydown} />
<main class="container">
  <SearchInput />
  <NoteList />
  <NoteView />

  <!-- Delete Confirmation Dialog -->
  {#if appState.showDeleteDialog}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dialog-overlay" onclick={closeDeleteDialog}>
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div
        class="dialog"
        bind:this={deleteDialogElement}
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
        <p>Are you sure you want to delete "{appState.selectedNote}"?</p>
        <p class="warning">This action cannot be undone.</p>
        <div class="keyboard-hint">
          <p>Press <kbd>DD</kbd> to confirm or <kbd>Esc</kbd> to cancel</p>
          {#if appState.deleteKeyPressCount === 1}
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
  {#if appState.showCreateDialog}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dialog-overlay" onclick={closeCreateDialog}>
      <div class="dialog" onclick={(e) => e.stopPropagation()}>
        <h3>Create New Note</h3>
        <input
          bind:this={appState.createDialogInputElement}
          bind:value={appState.newNoteName}
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
          <button class="btn-create" onclick={createNote} disabled={!appState.newNoteName.trim()}>Create</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Rename Note Dialog -->
  {#if appState.showRenameDialog}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dialog-overlay" onclick={closeRenameDialog}>
      <div class="dialog" onclick={(e) => e.stopPropagation()}>
        <h3>Rename Note</h3>
        <input
          bind:this={appState.renameDialogInputElement}
          bind:value={appState.newNoteNameForRename}
          placeholder="Enter new note name"
          class="note-name-input"
          onkeydown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              renameNote();
            } else if (e.key === 'Escape') {
              e.preventDefault();
              closeRenameDialog();
            }
          }}
        />
        <div class="dialog-buttons">
          <button class="btn-cancel" onclick={closeRenameDialog}>Cancel</button>
          <button class="btn-create" onclick={renameNote} disabled={!appState.newNoteNameForRename.trim()}>Rename</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Config Dialog -->
  {#if appState.showConfigDialog}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dialog-overlay" onclick={closeConfigDialog}>
      <div class="dialog config-dialog" onclick={(e) => e.stopPropagation()}>
        <h3>Configuration</h3>
        <div class="config-editor-container">
          <Editor
            bind:value={appState.configContent}
            filename="config.toml"
            onSave={saveConfig}
            onExit={closeConfigDialog}
          />
        </div>
        <div class="keyboard-hint">
          <p>Press <kbd>Ctrl+S</kbd> to save, <kbd>Esc</kbd> in normal mode to close</p>
        </div>
        <div class="dialog-buttons">
          <button class="btn-cancel" onclick={closeConfigDialog}>Cancel</button>
          <button class="btn-create" onclick={saveConfig}>Save</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Unsaved Changes Confirmation Dialog -->
  <ConfirmationDialog
    show={appState.showUnsavedChangesDialog}
    title="Unsaved Changes"
    message="You have unsaved changes. What would you like to do?"
    confirmText="Save and Exit"
    cancelText="Discard Changes"
    variant="default"
    on:confirm={handleSaveAndExit}
    on:cancel={handleDiscardAndExit}
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

.config-dialog {
  width: 900px;
  max-width: 90vw;
}

.config-editor-container {
  width: 100%;
  height: 500px;
  margin: 16px 0;
  border: 1px solid #504945;
  border-radius: 6px;
  overflow: hidden;
  background-color: #282828;
}
</style>
