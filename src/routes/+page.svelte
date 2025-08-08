<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMount, tick } from "svelte";
import { listen } from "@tauri-apps/api/event";
import SearchInput from "../lib/components/SearchInput.svelte";
import NoteList from "../lib/components/NoteList.svelte";
import NoteView from "../lib/components/NoteView.svelte";
import Editor from "../lib/components/Editor.svelte";
import ConfirmationDialog from "../lib/components/ConfirmationDialog.svelte";
import InputDialog from "../lib/components/InputDialog.svelte";
import DeleteDialog from "../lib/components/DeleteDialog.svelte";
import { createKeyboardHandler } from '../lib/keyboardHandler';
import { setAppContext } from '../lib/context/app.svelte';
import { contentHighlighter } from '../lib/utils/contentHighlighting.svelte';
import { searchManager } from '../lib/utils/searchManager.svelte';
import { dialogManager } from '../lib/utils/dialogManager.svelte';
import { noteService } from '../lib/services/noteService.svelte';
import { configService } from '../lib/services/configService.svelte';

interface SearchNotesResponse {
  [key: string]: string[];
}

type TauriInvokeResponse<T> = Promise<T>;

const appState = $state({
  searchInput: '',
  query: '',
  isLoading: false,
  areHighlightsCleared: false,

  filteredNotes: [] as string[],
  selectedNote: null as string | null,
  selectedIndex: -1,

  noteContent: '',
  highlightedContent: '',
  isEditMode: false,
  editContent: '',
  isEditorDirty: false,
  nearestHeaderText: '',

  isSearchInputFocused: false,
  isNoteContentFocused: false,
  searchElement: null as HTMLInputElement | null,
  noteListElement: null as HTMLElement | null,
  noteContentElement: null as HTMLElement | null,
});

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
  return await noteService.getContent(noteName);
}

async function deleteNote(): Promise<void> {
  if (!appState.selectedNote) return;

  await noteService.delete(
    appState.selectedNote,
    searchManager,
    dialogManager,
    (notes) => { appState.filteredNotes = notes; },
    appState.searchInput,
    () => appState.searchElement?.focus()
  );
}

async function createNote(noteNameParam?: string): Promise<void> {
  const inputNoteName = noteNameParam || dialogManager.newNoteName.trim();
  if (!inputNoteName.trim()) return;

  const finalNoteName = await noteService.create(
    inputNoteName,
    searchManager,
    dialogManager,
    (notes) => {
      appState.filteredNotes = notes;
      // Select the new note
      const noteIndex = notes.findIndex(note => note === (inputNoteName.includes('.') ? inputNoteName : `${inputNoteName}.md`));
      if (noteIndex >= 0) {
        appState.selectedIndex = noteIndex;
      }
    },
    () => appState.searchElement?.focus()
  );
}

async function renameNote(newNameParam?: string): Promise<void> {
  const inputNewName = newNameParam || dialogManager.newNoteNameForRename.trim();
  if (!inputNewName.trim() || !appState.selectedNote) return;

  await noteService.rename(
    appState.selectedNote,
    inputNewName,
    searchManager,
    dialogManager,
    (notes) => { appState.filteredNotes = notes; },
    (noteName) => {
      const noteIndex = appState.filteredNotes.findIndex(note => note === noteName);
      if (noteIndex >= 0) {
        appState.selectedIndex = noteIndex;
      }
    },
    appState.searchInput
  );
}

async function openSettingsPane(): Promise<void> {
  await configService.open(() => appState.searchElement?.focus());
}

function closeSettingsPane(): void {
  configService.close(() => appState.searchElement?.focus());
}

async function saveConfig(): Promise<void> {
  await configService.save(
    searchManager,
    (notes) => { appState.filteredNotes = notes; },
    () => appState.searchElement?.focus()
  );
}

function clearHighlights(): void {
  appState.areHighlightsCleared = true;
  appState.highlightedContent = contentHighlighter.highlighted;
}

$effect(() => {
  searchManager.updateSearchInputWithEffects(
    appState.searchInput,
    (query) => {
      appState.query = query;
    },
    (cleared) => {
      appState.areHighlightsCleared = cleared;
    }
  );
});

$effect(() => {
  appState.isLoading = searchManager.isLoading;
});

function arraysEqual(a: string[], b: string[]): boolean {
  return a.length === b.length && a.every((val, i) => val === b[i]);
}

$effect(() => {
  const notes = searchManager.filteredNotes;
  if (!arraysEqual(notes, appState.filteredNotes)) {
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
  if (!appState.selectedNote) {
    appState.noteContent = '';
    appState.highlightedContent = '';
    return;
  }

  if (contentRequestController) {
    contentRequestController.abort();
  }
  contentRequestController = new AbortController();
  const currentController = contentRequestController;

  (async () => {
    try {
      const content = await getNoteContent(appState.selectedNote!);

      if (!currentController.signal.aborted) {
        appState.noteContent = content;
        appState.highlightedContent = contentHighlighter.highlighted;

        requestAnimationFrame(() => {
          scrollToFirstMatch();
        });
      }
    } catch (e) {
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

$effect(() => {
  dialogManager.updateState({
    selectedNote: appState.selectedNote,
    query: appState.query,
    highlightedContent: appState.highlightedContent,
    searchElement: appState.searchElement
  });
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

async function saveNote(): Promise<void> {
  if (!appState.selectedNote || !appState.editContent) return;
  try {
    await invoke<void>("save_note", {
      noteName: appState.selectedNote,
      content: appState.editContent
    });

    await invoke<void>("refresh_cache");

    const notes = await searchManager.searchImmediate(appState.searchInput);
    appState.filteredNotes = notes;

    const content = await getNoteContent(appState.selectedNote);
    appState.noteContent = content;
    appState.highlightedContent = contentHighlighter.highlighted;
    appState.isEditMode = false;

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
    isEditorDirty: appState.isEditorDirty,
  }),
  {
    setSelectedIndex: (value: number) => appState.selectedIndex = value,
    enterEditMode,
    exitEditMode,
    showExitEditDialog: dialogManager.showExitEditDialog,
    saveNote,
    invoke,
    showDeleteDialog: () => dialogManager.openDeleteDialog(),
    showCreateDialog: () => dialogManager.openCreateDialog(),
    showRenameDialog: () => dialogManager.openRenameDialog(),
    clearHighlights,
    clearSearch: searchManager.clearSearch,
  }
);

setAppContext({
  state: appState,

  selectNote,
  deleteNote,
  createNote,
  renameNote,
  saveNote,
  enterEditMode,
  exitEditMode,
  showExitEditDialog: dialogManager.showExitEditDialog,
  handleSaveAndExit: () => dialogManager.handleSaveAndExit(saveNote, exitEditMode),
  handleDiscardAndExit: () => dialogManager.handleDiscardAndExit(exitEditMode),
  openCreateDialog: dialogManager.openCreateDialog,
  closeCreateDialog: dialogManager.closeCreateDialog,
  openRenameDialog: dialogManager.openRenameDialog,
  closeRenameDialog: dialogManager.closeRenameDialog,
  openDeleteDialog: dialogManager.openDeleteDialog,
  closeDeleteDialog: dialogManager.closeDeleteDialog,
  openSettingsPane,
  closeSettingsPane,
  saveConfig,
  handleDeleteKeyPress: () => dialogManager.handleDeleteKeyPress(() => deleteNote()),
  clearHighlights,
  clearSearch: searchManager.clearSearch,
  invoke,
});

onMount(() => {
  (async () => {
    await tick();

    const configExists = await invoke<boolean>("config_exists");
    if (!configExists) {
      openSettingsPane();
    } else {
      appState.searchElement?.focus();
      const notes = await searchManager.searchImmediate('');
      appState.filteredNotes = notes;
    }

    const unlisten = await listen("open-preferences", () => {
      openSettingsPane();
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

  <!-- Settings Pane -->
  {#if configService.isVisible}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dialog-overlay" onclick={closeSettingsPane}>
      <div class="dialog settings-pane" onclick={(e) => e.stopPropagation()}>
        <h3>Settings</h3>
        <div class="settings-editor-container">
          <Editor
            bind:value={configService.content}
            filename="config.toml"
            onSave={saveConfig}
            onExit={closeSettingsPane}
          />
        </div>
        <div class="keyboard-hint">
          <p>Press <kbd>Ctrl+S</kbd> to save, <kbd>Esc</kbd> in normal mode to close</p>
        </div>
        <div class="dialog-buttons">
          <button class="btn-cancel" onclick={closeSettingsPane}>Cancel</button>
          <button class="btn-create" onclick={saveConfig}>Save</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Delete Confirmation Dialog -->
  <DeleteDialog
    show={dialogManager.showDeleteDialog}
    noteName={appState.selectedNote || ''}
    deleteKeyPressCount={dialogManager.deleteKeyPressCount}
    onConfirm={deleteNote}
    onCancel={dialogManager.closeDeleteDialog}
    onKeyPress={() => dialogManager.handleDeleteKeyPress(() => deleteNote())}
  />

  <!-- Create Note Dialog -->
  <InputDialog
    show={dialogManager.showCreateDialog}
    title="Create New Note"
    value={dialogManager.newNoteName}
    placeholder="Enter note name (extension will be .md)"
    confirmText="Create"
    cancelText="Cancel"
    onConfirm={(value) => createNote(value)}
    onCancel={dialogManager.closeCreateDialog}
    onInput={(value) => dialogManager.setNewNoteName(value)}
  />

  <!-- Rename Note Dialog -->
  <InputDialog
    show={dialogManager.showRenameDialog}
    title="Rename Note"
    value={dialogManager.newNoteNameForRename}
    placeholder="Enter new note name"
    confirmText="Rename"
    cancelText="Cancel"
    autoSelect={true}
    onConfirm={(value) => renameNote(value)}
    onCancel={dialogManager.closeRenameDialog}
    onInput={(value) => dialogManager.setNewNoteNameForRename(value)}
  />

  <!-- Unsaved Changes Confirmation Dialog -->
  <ConfirmationDialog
    show={dialogManager.showUnsavedChangesDialog}
    title="Unsaved Changes"
    message="You have unsaved changes. What would you like to do?"
    confirmText="Save and Exit"
    cancelText="Discard Changes"
    variant="default"
    onConfirm={() => dialogManager.handleSaveAndExit(saveNote, exitEditMode)}
    onCancel={() => dialogManager.handleDiscardAndExit(exitEditMode)}
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

.settings-pane {
  width: 900px;
  max-width: 90vw;
}

.settings-editor-container {
  width: 100%;
  height: 500px;
  margin: 16px 0;
  border: 1px solid #504945;
  border-radius: 6px;
  overflow: hidden;
  background-color: #282828;
}
</style>
